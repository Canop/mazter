use {
    crate::*,
    anyhow::anyhow,
    fnv::FnvHasher,
    std::{
        fs,
        hash::{Hash, Hasher},
        path::PathBuf,
    },
};

const SALT: u64 = 20220722;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Achievement<'s> {
    user: &'s str,
    level: usize,
}

impl<'s> Achievement<'s> {
    pub fn new(user: &'s str, level:usize) -> Self {
        Self { user, level }
    }
    /// get the hash according to FNV
    pub fn hash(self) -> u64 {
        let mut hasher = FnvHasher::with_key(SALT);
        self.user.hash(&mut hasher);
        let specs = Specs::for_level(self.level);
        specs.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Record {
    user: String,
    level: usize,
    hash: u64,
}

impl<'s> From<Achievement<'s>> for Record {
    fn from(ach: Achievement<'s>) -> Self {
        let level = ach.level;
        let user = ach.user.to_string();
        let hash = ach.hash();
        Self { user, level, hash }
    }
}

impl Record {
    pub fn achievement(&self) -> Achievement<'_> {
        Achievement::new(&self.user, self.level)
    }
    pub fn is_valid(&self) -> bool {
        self.hash == self.achievement().hash()
    }
}

/// Achievement Database
pub struct Database {
    file_path: PathBuf,
    records: Vec<Record>,
}

impl Database {
    fn new() -> anyhow::Result<Self> {
        let project_dirs = directories::ProjectDirs::from("org", "dystroy", "mazter")
            .ok_or_else(|| anyhow!("no conf directory"))?;
        let file_path = project_dirs.data_dir().join("achievements.csv");
        debug!("file_path: {:?}", &file_path);
        let mut records = Vec::new();
        if file_path.exists() {
            let mut csv_reader = csv::Reader::from_path(&file_path)?;
            for res in csv_reader.deserialize() {
                let record: Record = res?;
                if record.is_valid() {
                    records.push(record);
                } else {
                    // the most normal cause is the level spec having changed
                    // since the user won it
                    info!("invalid record: {:#?}", &record);
                }
            }
        }
        Ok(Self { file_path, records })
    }
    fn add(&mut self, ach: Achievement) {
        self.records.push(ach.into());
    }
    fn write(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.file_path.parent().expect("conf file parent should be defined"))?;
        let mut writer = csv::Writer::from_path(&self.file_path)?;
        for record in &self.records {
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }
    fn contains(&self, ach: Achievement) -> bool {
        self.records
            .iter()
            .any(|record| record.achievement() == ach)
    }

    pub fn save(ach: Achievement) -> anyhow::Result<()> {
        let mut db = Self::new()?;
        db.add(ach);
        db.write()?;
        Ok(())
    }
    /// save the achievement and return the first following
    /// level not won
    pub fn advance(ach: Achievement) -> anyhow::Result<usize> {
        let mut db = Self::new()?;
        db.add(ach);
        db.write()?;
        let mut level = ach.level + 1;
        loop {
            if !db.contains(Achievement::new(ach.user, level)) {
                return Ok(level);
            }
            level += 1;
        }
    }
    pub fn first_not_won(user: &str) -> anyhow::Result<usize> {
        let db = Self::new()?;
        let mut level = 1;
        loop {
            if !db.contains(Achievement::new(user, level)) {
                return Ok(level);
            }
            level += 1;
        }
    }
    pub fn can_play(user: &str, target: usize) -> anyhow::Result<bool> {
        if target == 0 {
            return Ok(true);
        }
        let db = Self::new()?;
        let mut level = 1;
        loop {
            if level == target {
                return Ok(true);
            }
            if !db.contains(Achievement::new(user, level)) {
                return Ok(false);
            }
            level += 1;
        }
    }
    pub fn reset(user: &str, print: bool) -> anyhow::Result<()> {
        let mut db = Self::new()?;
        if db.records.iter().any(|record| record.user == user) {
            if print {
                println!("Removing achievements of user {user:?}");
                println!("If you change your mind, you can put the folowwing lines back in {:?}", db.file_path);
            }
            let mut records = Vec::new();
            let mut printer = csv::Writer::from_writer(std::io::stdout());
            for record in db.records.drain(..) {
                if record.user == user {
                    if print {
                        printer.serialize(record)?;
                    }
                } else {
                    records.push(record);
                }
            }
            db.records = records;
            db.write()?;
        } else {
            if print {
                println!("No achievement were found for user {user:?}");
            }
        }
        Ok(())
    }
}
