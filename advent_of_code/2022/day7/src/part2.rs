use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::BinaryHeap,
    env::current_dir,
    rc::{Rc, Weak},
};

#[derive(Debug, Clone)]
struct File {
    name: String,
    parent: Weak<RefCell<Directory>>,
    size: u32,
}

#[derive(Debug, Clone)]
struct Directory {
    name: String,
    parent: Option<Weak<RefCell<Directory>>>,
    files: Vec<File>,
    directories: Vec<Rc<RefCell<Directory>>>,
}

impl Directory {
    fn new(name: &str, parent: Option<Weak<RefCell<Directory>>>) -> Directory {
        Directory {
            name: name.to_string(),
            parent,
            files: Vec::new(),
            directories: Vec::new(),
        }
    }

    fn add_file(&mut self, name: &str, size: u32, root: &Rc<RefCell<Directory>>) {
        let file = File {
            name: name.to_string(),
            parent: Rc::downgrade(root),
            size,
        };
        self.files.push(file);
        // Remove: self.size += size;
    }

    fn add_dir(&mut self, name: &str, root: &Rc<RefCell<Directory>>) -> Rc<RefCell<Directory>> {
        let new_dir = Rc::new(RefCell::new(Directory::new(
            name,
            Some(Rc::downgrade(root)),
        )));
        self.directories.push(Rc::clone(&new_dir));
        // Remove: self.size += new_dir.borrow().size;
        new_dir
    }

    fn get_size(&self) -> u32 {
        let files_size = self.files.iter().map(|f| f.size).sum::<u32>();
        let dirs_size = self
            .directories
            .iter()
            .map(|d| d.borrow().get_size())
            .sum::<u32>();
        files_size + dirs_size
    }

    fn sum_directories_at_most(&self, max_size: u32) -> u32 {
        let mut sum = 0;
        let size = self.get_size();
        if size <= max_size {
            sum += size;
        }

        for dir in &self.directories {
            sum += dir.borrow().sum_directories_at_most(max_size);
        }
        sum
    }

    fn change_dir(
        current_dir: &mut Rc<RefCell<Directory>>,
        target: &str,
        root: &Rc<RefCell<Directory>>,
    ) -> Result<Rc<RefCell<Directory>>, ()> {
        if target == ".." {
            if let Some(parent_weak) = &current_dir.borrow().parent {
                if let Some(parent) = parent_weak.upgrade() {
                    return Ok(parent);
                }
            }
            Err(())
        } else {
            let mut dir = if target.starts_with('/') {
                Rc::clone(root)
            } else {
                Rc::clone(current_dir)
            };

            let paths = target.split('/').collect::<Vec<_>>();

            for path in paths {
                let mut found = false;
                let directories = dir.borrow().directories.clone();
                for d in directories {
                    if d.borrow().name == path {
                        dir = Rc::clone(&d);
                        found = true;
                    }
                }
                if !found {
                    let new_dir = dir.borrow_mut().add_dir(path, root);
                    dir = new_dir;
                }
            }

            Ok(dir)
        }
    }

    fn get_all_directories(dir: Rc<RefCell<Directory>>) -> Vec<Rc<RefCell<Directory>>> {
        let mut dirs = vec![Rc::clone(&dir)];
        for sub_dir in &dir.borrow().directories {
            dirs.extend(Directory::get_all_directories(Rc::clone(sub_dir)));
        }
        dirs
    }
}

pub fn solve(input: &str) -> u32 {
    let lines = input.lines().collect::<Vec<_>>();

    let root = Rc::new(RefCell::new(Directory::new("/", None)));
    let mut current_dir = Rc::clone(&root);

    let mut is_ls = false;
    for line in lines {
        println!("{}", line);
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.is_empty() {
            continue;
        }

        if parts[0] == "$" {
            is_ls = false;
            if parts.len() == 1 {
                continue;
            }

            let cmd = parts[1];
            match cmd {
                "cd" => {
                    if parts.len() < 3 {
                        continue;
                    }
                    let target = parts[2];
                    if let Ok(dir) = Directory::change_dir(&mut current_dir, target, &root) {
                        current_dir = dir;
                    }
                }

                "ls" => {
                    is_ls = true;
                    continue;
                }
                _ => {}
            }
        } else if is_ls {
            if parts.len() < 2 {
                continue;
            }

            if parts[0] == "dir" {
                let name = parts[1];
                current_dir.borrow_mut().add_dir(name, &current_dir);
            } else {
                if let Ok(size) = parts[0].parse::<u32>() {
                    let name = parts[1];
                    current_dir.borrow_mut().add_file(name, size, &current_dir);
                }
            }
        }
    }

    let root_borrowed = root.borrow();
    let current_root_size = root_borrowed.get_size();
    const SIZE_NEEDED_FOR_UPDATE: u32 = 30000000;
    const SYSTEM_SIZE: u32 = 70000000;
    let available_size = SYSTEM_SIZE - current_root_size;
    let size_to_obtain = SIZE_NEEDED_FOR_UPDATE - available_size;
    println!("available_size: {}", available_size);
    println!("current_root_size: {}", current_root_size);
    println!("size_to_obtain: {}", size_to_obtain);

    let mut dirs = Directory::get_all_directories(Rc::clone(&root));
    dirs.sort_by_key(|b| b.borrow().get_size());

    println!(
        "dirs: {:#?}",
        dirs.iter()
            .map(|d| d.borrow().get_size())
            .collect::<Vec<_>>()
    );
    let mut size = current_root_size;

    for dir in dirs {
        size = dir.borrow().get_size();
        if size >= size_to_obtain {
            return size;
        }
    }
    size
}
