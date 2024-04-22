use std::str::FromStr;
use std::{collections::HashSet, path::PathBuf};

use crate::model::reference::ActiveModel;
use crate::model::{config, reference};
use crate::{db::establish_connection, internal::index::Index, utils::util};
use clap::Parser;
use sea_orm::{ActiveModelTrait, Set};
use storage::driver::file_storage::{local_storage::LocalStorage, FileStorage};
use venus::hash::SHA1;
use venus::internal::object::commit::Commit;
use venus::internal::object::tree::{Tree, TreeItem, TreeItemMode};

#[derive(Parser, Debug)]
#[command(about = "Record changes to the repository")]
pub struct CommitArgs {
    #[arg(short, long)]
    pub message: String,

    #[arg(long)]
    pub allow_empty: bool,
}

pub async fn execute(args: CommitArgs) {
    /* check args */
    let index = Index::from_file(util::working_dir().join("index")).unwrap();
    let storage = LocalStorage::init(util::storage_path().join("objects"));
    let tracked_entries = index.tracked_entries(0);
    if tracked_entries.is_empty() && !args.allow_empty {
        panic!("fatal: no changes added to commit, use --allow-empty to override");
    }

    /* Create tree */
    let tree = create_tree(&index, &storage, "".into()).await;
    // TODO wait for head & status
    let db = establish_connection(
        util::path_to_string(&util::storage_path().join(util::DATABASE)).as_str(),
    )
    .await
    .unwrap();

    /* Create & save commit objects */
    let parents_commit_ids = get_parents_ids(&db).await;
    let commit = Commit::from_tree_id(tree.id, parents_commit_ids, args.message.as_str());

    // TODO  default signature created in `frrom_tree_id`, wait `git config` to set correct user info

    storage
        .put(
            &commit.id.to_plain_str(),
            commit.to_data().unwrap().len() as i64,
            &commit.to_data().unwrap(),
        )
        .await
        .unwrap();

    /* update HEAD */
    update_head(&db, &commit.id.to_plain_str()).await;

    // TODO make some test
}

/// recursively create tree from index's tracked entries
async fn create_tree(index: &Index, storage: &dyn FileStorage, current_root: PathBuf) -> Tree {
    // blob created when add file to index
    let get_blob_entry = |path: &PathBuf| {
        let name = util::path_to_string(path);
        let mete = index.get(&name, 0).unwrap();
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();

        TreeItem {
            name: filename,
            mode: TreeItemMode::tree_item_type_from_bytes(format!("{:o}", mete.mode).as_bytes())
                .unwrap(),
            id: mete.hash,
        }
    };

    let mut tree_items: Vec<TreeItem> = Vec::new();
    let mut processed_path: HashSet<String> = HashSet::new();
    let path_entries: Vec<PathBuf> = index
        .tracked_entries(0)
        .iter()
        .map(|file| PathBuf::from(file.name.clone()))
        .filter(|path| path.starts_with(&current_root))
        .collect();
    for path in path_entries.iter() {
        // check if the file is in the current root
        let in_path = path.parent().unwrap() == current_root;
        if in_path {
            let item = get_blob_entry(path);
            tree_items.push(item);
        } else {
            if path.components().count() == 1 {
                continue;
            }
            // 拿到下一级别目录
            let process_path = path
                .components()
                .nth(current_root.components().count())
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap();

            if processed_path.contains(process_path) {
                continue;
            }
            processed_path.insert(process_path.to_string());

            let sub_tree = Box::pin(create_tree(
                index,
                storage,
                current_root.clone().join(process_path),
            ))
            .await;
            tree_items.push(TreeItem {
                name: process_path.to_string(),
                mode: TreeItemMode::Tree,
                id: sub_tree.id,
            });
        }
    }
    let tree = Tree::from_tree_items(tree_items).unwrap();
    // save
    let data = tree.to_data().unwrap();
    storage
        .put(&tree.id.to_plain_str(), data.len() as i64, &data)
        .await
        .unwrap();
    tree
}

/// get current head commit id as parent, if in branch, get branch's commit id, if detached head, get head's commit id
async fn get_parents_ids(db: &sea_orm::DbConn) -> Vec<SHA1> {
    let head = reference::Model::current_head(db).await.unwrap();
    match head {
        Some(head) => match head.name {
            Some(name) => {
                let commit = reference::Model::find_branch_by_name(db, name.as_str())
                    .await
                    .unwrap()
                    .unwrap();
                vec![SHA1::from_str(commit.commit.unwrap().as_str()).unwrap()]
            }
            None => vec![SHA1::from_str(head.commit.unwrap().as_str()).unwrap()],
        },
        None => vec![],
    }
}

/// update HEAD to new commit, if in branch, update branch's commit id, if detached head, update head's commit id
async fn update_head(db: &sea_orm::DbConn, commit_id: &str) {
    let head = reference::Model::current_head(db).await.unwrap();
    match head {
        Some(head) => {
            match head.name {
                Some(name) => {
                    // in branch
                    let mut branch: ActiveModel =
                        reference::Model::find_branch_by_name(db, name.as_str())
                            .await
                            .unwrap()
                            .unwrap()
                            .into();
                    branch.commit = Set(Some(commit_id.to_string()));
                    branch.update(db).await.unwrap();
                }
                None => {
                    // detached head
                    let mut head: ActiveModel = head.into();
                    head.commit = Set(Some(commit_id.to_string()));
                    head.update(db).await.unwrap();
                }
            }
        }
        None => {
            // create main branch
            let branch = reference::ActiveModel {
                name: Set(Some("main".to_owned())),
                kind: Set(reference::ConfigKind::Branch),
                commit: Set(Some(commit_id.to_string())),
                ..Default::default()
            };
            branch.save(db).await.unwrap();

            // create & set head to main
            let head = reference::ActiveModel {
                name: Set(Some("main".to_owned())),
                kind: Set(reference::ConfigKind::Head),
                ..Default::default()
            };
            head.save(db).await.unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use venus::internal::object::ObjectTrait;

    use crate::utils::test;

    use super::*;

    #[tokio::test]
    async fn test_create_tree() {
        let index = Index::from_file(
            "../tests/data/index/index-760,
        timestamp: todo!(),
        timezone: todo!(), ",
        )
        .unwrap();
        println!("{:?}", index.tracked_entries(0).len());
        test::setup_with_new_libra().await;
        let storage = LocalStorage::init(util::storage_path().join("objects"));
        let tree = create_tree(&index, &storage, "".into()).await;

        assert!(storage.get(&tree.id.to_plain_str()).await.is_ok());
        for item in tree.tree_items.iter() {
            if item.mode == TreeItemMode::Tree {
                assert!(storage.get(&item.id.to_plain_str()).await.is_ok());
                // println!("tree: {}", item.name);
                if item.name == "DeveloperExperience" {
                    let sub_tree = storage.get(&item.id.to_plain_str()).await.unwrap();
                    let tree = Tree::from_bytes(sub_tree.to_vec(), item.id).unwrap();
                    assert!(tree.tree_items.len() == 4); // 4 sub tree according to the test data
                }
            }
        }
    }
}
