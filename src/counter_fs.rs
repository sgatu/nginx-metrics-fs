use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use fuser::{FileAttr, FileType, Filesystem};
use libc::ENOENT;
macro_rules! PRETTY_OUTPUT_TEMPLATE {
    () => {
        "|       DateTime       |    # HTTP 100    |    # HTTP 200    |    # HTTP 300    |    # HTTP 400    |    # HTTP 500    |
{}
"
    };
}
macro_rules! PARSEABLE_OUTPUT_TEMPLATE {
    () => {
        "|dt|HTTP_100|HTTP_200|HTTP_300|HTTP_400|HTTP_500|
{}
"
    };
}

pub struct Counters {
    moment: u32,
    stats100: u32,
    stats200: u32,
    stats300: u32,
    stats400: u32,
    stats500: u32,
}
impl Counters {
    fn new() -> Counters {
        Counters {
            moment: CounterFS::get_minute(SystemTime::now()),
            stats100: 0,
            stats200: 0,
            stats300: 0,
            stats400: 0,
            stats500: 0,
        }
    }
}
const INO_DIR: u64 = 1;
const INO_PRETTY_STATS: u64 = 2;
const INO_STATS: u64 = 3;
const DIR_STATS: FileAttr = FileAttr {
    ino: INO_DIR,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o777,
    nlink: 2,
    uid: 0,
    gid: 0,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

pub struct CounterFS {
    counters: Vec<Counters>,
    file_name: String,
    pretty_file_name: Option<String>,
    attr_file: FileAttr,
    attr_file_pretty: FileAttr,
    time_points: u16,
    regex_status_code: String,
}
impl CounterFS {
    pub fn new(
        file_name: String,
        has_pretty: bool,
        time_points: u16,
        regex_status_code: String,
    ) -> CounterFS {
        let mut pretty_file_name: Option<String> = None;
        if has_pretty {
            pretty_file_name = Some(("pretty_".to_owned() + &file_name).to_owned());
        }
        CounterFS {
            counters: vec![Counters::new()],
            file_name: file_name,
            pretty_file_name: pretty_file_name,
            attr_file: FileAttr {
                ino: INO_STATS,
                size: 0,
                blocks: 1,
                atime: SystemTime::now(), // 1970-01-01 00:00:00
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
                crtime: SystemTime::now(),
                kind: FileType::RegularFile,
                perm: 0o777,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: 0,
                blksize: 512,
            },
            attr_file_pretty: FileAttr {
                ino: INO_PRETTY_STATS,
                size: 0,
                blocks: 1,
                atime: SystemTime::now(), // 1970-01-01 00:00:00
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
                crtime: SystemTime::now(),
                kind: FileType::RegularFile,
                perm: 0o777,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: 0,
                blksize: 512,
            },
            time_points: time_points,
            regex_status_code: regex_status_code,
        }
    }
    fn get_stats_pretty(&self) -> String {
        let mut data: String = "".to_owned();
        for i in 0..self.counters.len() {
            let dt: DateTime<Utc> = UNIX_EPOCH
                .checked_add(Duration::from_secs(self.counters[i].moment as u64))
                .unwrap()
                .into();
            if data.len() > 0 {
                data = data + "\n";
            }
            data = data
                + format!(
                    "|{: ^22}|{: ^18}|{: ^18}|{: ^18}|{: ^18}|{: ^18}|",
                    dt.format("%d-%m-%Y %T"),
                    &self.counters[i].stats100,
                    &self.counters[i].stats200,
                    &self.counters[i].stats300,
                    &self.counters[i].stats400,
                    &self.counters[i].stats500
                )
                .as_str();
        }
        let full = format!(PRETTY_OUTPUT_TEMPLATE!(), data);
        return full;
    }

    fn get_stats_parseable(&self) -> String {
        let mut data: String = "".to_owned();
        for i in 0..self.counters.len() {
            let dt: DateTime<Utc> = UNIX_EPOCH
                .checked_add(Duration::from_secs(self.counters[i].moment as u64))
                .unwrap()
                .into();
            if data.len() > 0 {
                data = data + "\n";
            }
            data = data
                + format!(
                    "|{}|{}|{}|{}|{}|{}|",
                    dt.format("%d-%m-%Y %T"),
                    &self.counters[i].stats100,
                    &self.counters[i].stats200,
                    &self.counters[i].stats300,
                    &self.counters[i].stats400,
                    &self.counters[i].stats500
                )
                .as_str();
        }
        let full = format!(PARSEABLE_OUTPUT_TEMPLATE!(), data);
        return full;
    }
    fn get_minute(when: SystemTime) -> u32 {
        return u32::try_from((when.duration_since(UNIX_EPOCH).unwrap().as_secs() / 60) * 60)
            .unwrap();
    }
    fn push_status(&mut self, code: &str) {
        self.attr_file.mtime = SystemTime::now();
        self.attr_file_pretty.mtime = self.attr_file.mtime;
        let minute = Self::get_minute(self.attr_file.mtime);
        if let Some(last_counter) = self.counters.last_mut() {
            if last_counter.moment == minute {
                match code.as_bytes()[0] {
                    b'1' => last_counter.stats100 += 1,
                    b'2' => last_counter.stats200 += 1,
                    b'3' => last_counter.stats300 += 1,
                    b'4' => last_counter.stats400 += 1,
                    b'5' => last_counter.stats500 += 1,
                    _ => {}
                }
                return;
            }
        }
        let mut counter = Counters::new();
        match code.as_bytes()[0] {
            b'1' => counter.stats100 += 1,
            b'2' => counter.stats200 += 1,
            b'3' => counter.stats300 += 1,
            b'4' => counter.stats400 += 1,
            b'5' => counter.stats500 += 1,
            _ => {}
        }
        self.counters.push(counter);
        if self.counters.len() > self.time_points as usize {
            let _ = self.counters.remove(0);
        }
    }
}
impl Filesystem for CounterFS {
    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        let data = match ino {
            INO_PRETTY_STATS if self.pretty_file_name.is_some() => self.get_stats_pretty(),
            INO_STATS => self.get_stats_parseable(),
            _ => {
                reply.error(ENOENT);
                return;
            }
        };
        reply.data(&data.as_bytes()[offset as usize..]);
    }
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        if parent == INO_DIR {
            match name.to_str() {
                Some(m) if m == &self.file_name => {
                    let stats = self.get_stats_parseable();
                    self.attr_file.size = stats.len() as u64;
                    self.attr_file.blocks = (stats.len() as f64 / 512 as f64).ceil() as u64;
                    reply.entry(&Duration::from_secs(1), &self.attr_file, 0);
                }
                Some(m) => {
                    if let Some(p) = &self.pretty_file_name {
                        if p.as_str() == m {
                            let stats = self.get_stats_pretty();
                            self.attr_file_pretty.size = stats.len() as u64;
                            self.attr_file_pretty.blocks =
                                (stats.len() as f64 / 512 as f64).ceil() as u64;
                            reply.entry(&Duration::from_secs(1), &self.attr_file_pretty, 0);
                            return;
                        }
                    }
                    reply.error(ENOENT);
                }
                _ => reply.error(ENOENT),
            }
        } else {
            reply.error(ENOENT)
        }
    }
    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        match ino {
            INO_DIR => reply.attr(&Duration::from_secs(1), &DIR_STATS),
            INO_PRETTY_STATS if self.pretty_file_name.is_some() => {
                let stats = self.get_stats_pretty();
                self.attr_file_pretty.size = stats.len() as u64;
                self.attr_file_pretty.blocks = (stats.len() as f64 / 512 as f64).ceil() as u64;
                reply.attr(&Duration::from_secs(1), &self.attr_file_pretty)
            }
            INO_STATS => {
                let stats = self.get_stats_parseable();
                self.attr_file.size = stats.len() as u64;
                self.attr_file.blocks = (stats.len() as f64 / 512 as f64).ceil() as u64;
                reply.attr(&Duration::from_secs(1), &self.attr_file)
            }
            _ => reply.error(ENOENT),
        }
    }

    fn write(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        _offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyWrite,
    ) {
        if ino != INO_STATS {
            reply.error(ENOENT);
            return;
        }
        let str_data = String::from_utf8_lossy(data);
        let regx = regex::Regex::new(&self.regex_status_code).unwrap();
        let _matches = regx.captures(&str_data);
        if let Some(captures) = _matches {
            if captures.len() > 1 {
                if let Some(capt_code) = captures.get(1) {
                    self.push_status(capt_code.as_str())
                }
            } else {
                println!("No capture");
            }
        } else {
            println!("No capture");
        }
        reply.written(data.len() as u32)
    }
    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }
        let mut entries = vec![
            (INO_DIR, FileType::Directory, "."),
            (INO_DIR, FileType::Directory, ".."),
            (INO_STATS, FileType::RegularFile, &self.file_name),
        ];
        if let Some(p) = &self.pretty_file_name {
            entries.push((INO_PRETTY_STATS, FileType::RegularFile, p.as_str()));
        }
        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }
        reply.ok();
    }
}
