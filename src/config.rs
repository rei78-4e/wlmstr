use serde::{Deserialize, Serialize};

/// placeholder replaced with the wallpaper path in apply commands
pub const PATH_PLACEHOLDER: &str = "{path}";

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub apply: Apply,

    /// deprecated: use `[apply] image` instead
    pub awww: Option<Awww>,
}

impl easy_storage::Storeable for Config {}

/// commands used to set wallpapers. `{path}` is replaced with the wallpaper path.
/// if no argument contains `{path}`, the path is appended to the end.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Apply {
    pub image: Option<Vec<String>>,
    pub video: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Awww {
    pub args: Vec<String>,
}

impl Config {
    pub fn image_cmd(&self) -> Vec<String> {
        if let Some(cmd) = &self.apply.image {
            return cmd.clone();
        }
        let extra = match &self.awww {
            Some(awww) => awww.args.clone(),
            None => [
                "--transition-type",
                "center",
                "--transition-duration",
                "0.5",
            ]
            .map(String::from)
            .to_vec(),
        };
        [
            ["awww", "img", PATH_PLACEHOLDER].map(String::from).to_vec(),
            extra,
        ]
        .concat()
    }

    pub fn video_cmd(&self) -> Vec<String> {
        if let Some(cmd) = &self.apply.video {
            return cmd.clone();
        }
        [
            "mpvpaper",
            "*",
            PATH_PLACEHOLDER,
            "-o",
            "no-audio loop",
            "--fork",
            "-o",
            "no-audio loop --panscan=1.0",
            "--mpv-args=\"--hwdec=auto-safe --panscan=1.0\"",
        ]
        .map(String::from)
        .to_vec()
    }
}

/// expand `{path}` in `template`. appends `path` when no placeholder exists.
pub fn expand_cmd(template: &[String], path: &str) -> Vec<String> {
    let has_placeholder = template.iter().any(|a| a.contains(PATH_PLACEHOLDER));
    let mut args: Vec<String> = template
        .iter()
        .map(|a| a.replace(PATH_PLACEHOLDER, path))
        .collect();
    if !has_placeholder {
        args.push(path.to_string());
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn expand_replaces_placeholder() {
        let t = s(&["awww", "img", "{path}", "--x"]);
        assert_eq!(expand_cmd(&t, "/a b.png"), s(&["awww", "img", "/a b.png", "--x"]));
    }

    #[test]
    fn expand_appends_without_placeholder() {
        let t = s(&["swaybg", "-i"]);
        assert_eq!(expand_cmd(&t, "/a.png"), s(&["swaybg", "-i", "/a.png"]));
    }

    #[test]
    fn legacy_awww_args_are_used() {
        let c = Config {
            awww: Some(Awww { args: s(&["--transition-type", "wave"]) }),
            ..Default::default()
        };
        assert_eq!(
            c.image_cmd(),
            s(&["awww", "img", "{path}", "--transition-type", "wave"])
        );
    }

    #[test]
    fn apply_image_overrides_legacy() {
        let c = Config {
            apply: Apply { image: Some(s(&["swaybg", "-i"])), video: None },
            awww: Some(Awww { args: s(&["--x"]) }),
        };
        assert_eq!(c.image_cmd(), s(&["swaybg", "-i"]));
    }
}
