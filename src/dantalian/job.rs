use super::config::Config;
use super::utils::is_video_file;
use anyhow::{anyhow, Result};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub struct Job {
    pub subject_id: u32,
    pub should_gen_tvshow: bool,
    pub episodes: Vec<EpisodeJob>,
}

pub struct EpisodeJob {
    pub index: String,
    pub is_sp: bool,
    pub filename: String,
}

const TVSHOW_NFO_NAME: &str = "tvshow.nfo";

impl Job {
    pub fn parse(dir: &Path, config: &Config, force: bool) -> Result<Job> {
        let tv_show_file = dir.join(TVSHOW_NFO_NAME);
        let should_gen_tvshow = force || !tv_show_file.exists();
        let mut episodes: Vec<EpisodeJob> = vec![];
        for e in WalkDir::new(dir).min_depth(1).max_depth(1) {
            let entry = e?;
            if entry.file_type().is_file() {
                let ep = Self::check_episode(&entry, config, force)?;
                if let Some(ep_job) = ep {
                    episodes.push(ep_job);
                }
            }
        }
        Ok(Job {
            subject_id: config.subject_id,
            should_gen_tvshow,
            episodes,
        })
    }

    fn check_episode(
        file_entry: &DirEntry,
        config: &Config,
        force: bool,
    ) -> Result<Option<EpisodeJob>> {
        if !is_video_file(file_entry.path()) {
            // if this file is not video file, skip it.
            return Ok(None);
        }
        let Some(file_name) = file_entry.file_name().to_str() else {
            return Ok(None);
        };
        let nfo_file_path = file_entry.path().with_extension("nfo");
        if (!force) && nfo_file_path.exists() {
            // nfo file of current file already exists, don't need a job
            return Ok(None);
        }
        let caps = config.episode_re.captures(file_name);
        let Some(matched_ep) = caps.as_ref().and_then(|c| c.name("ep")) else {
            return Ok(None);
        };
        let ep_str = match matched_ep.as_str().trim_start_matches('0') {
            "" => "0",
            ep_str => ep_str,
        };
        let ep = match config.episode_offset {
            0 => ep_str.to_string(),
            offset => ep_str
                .parse::<f64>()
                .map_or_else(|s| s.to_string(), |e| (e + offset as f64).to_string()),
        };
        let sp = caps
            .and_then(|c| c.name("sp"))
            .map_or(false, |mat| mat.as_str() != "");
        Ok(Some(EpisodeJob {
            index: ep,
            is_sp: sp,
            filename: String::from(
                nfo_file_path
                    .to_str()
                    .ok_or_else(|| anyhow!("invalid nfo file name"))?,
            ),
        }))
    }

    pub fn is_empty(&self) -> bool {
        (!self.should_gen_tvshow) && (self.episodes.len() == 0)
    }
}
