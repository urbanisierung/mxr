//! Build invocations for the official deploy CLIs (wrangler, C3, flyctl) and
//! `gh secret set`. The functions here only assemble argument vectors so they
//! stay pure and testable; the CLI crate is responsible for actually spawning
//! the commands.

/// GitHub Actions secrets the Cloudflare deploy workflow expects.
pub const CLOUDFLARE_SECRETS: &[&str] = &["CLOUDFLARE_API_TOKEN", "CLOUDFLARE_ACCOUNT_ID"];

/// GitHub Actions secrets the Fly.io deploy workflow expects.
pub const FLY_SECRETS: &[&str] = &["FLY_API_TOKEN"];

/// `wrangler` args to create a Cloudflare Pages project to deploy to.
pub fn cloudflare_pages_args(name: &str, branch: &str) -> Vec<String> {
    vec![
        "pages".into(),
        "project".into(),
        "create".into(),
        name.into(),
        "--production-branch".into(),
        branch.into(),
    ]
}

/// `npm` args to scaffold a Cloudflare Worker project with C3.
pub fn cloudflare_worker_args(name: &str) -> Vec<String> {
    vec!["create".into(), "cloudflare@latest".into(), name.into()]
}

/// `fly` args to create a Fly.io app to deploy to. A generated name is used
/// when none is given.
pub fn fly_create_args(name: Option<&str>) -> Vec<String> {
    match name {
        Some(n) => vec!["apps".into(), "create".into(), n.into()],
        None => vec!["apps".into(), "create".into(), "--generate-name".into()],
    }
}

/// `gh` args to set a repository Actions secret. With no value, `gh` reads the
/// secret from stdin (or prompts when interactive).
pub fn gh_secret_set_args(key: &str, value: Option<&str>) -> Vec<String> {
    match value {
        Some(v) => vec![
            "secret".into(),
            "set".into(),
            key.into(),
            "--body".into(),
            v.into(),
        ],
        None => vec!["secret".into(), "set".into(), key.into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_args_include_name_and_branch() {
        assert_eq!(
            cloudflare_pages_args("mxrlp", "main"),
            [
                "pages",
                "project",
                "create",
                "mxrlp",
                "--production-branch",
                "main"
            ]
        );
    }

    #[test]
    fn worker_args_invoke_c3() {
        assert_eq!(
            cloudflare_worker_args("my-worker"),
            ["create", "cloudflare@latest", "my-worker"]
        );
    }

    #[test]
    fn fly_args_with_name() {
        assert_eq!(fly_create_args(Some("myapp")), ["apps", "create", "myapp"]);
    }

    #[test]
    fn fly_args_without_name_generates() {
        assert_eq!(fly_create_args(None), ["apps", "create", "--generate-name"]);
    }

    #[test]
    fn secret_args_with_value_use_body() {
        assert_eq!(
            gh_secret_set_args("FLY_API_TOKEN", Some("tok")),
            ["secret", "set", "FLY_API_TOKEN", "--body", "tok"]
        );
    }

    #[test]
    fn secret_args_without_value_read_stdin() {
        assert_eq!(
            gh_secret_set_args("CLOUDFLARE_API_TOKEN", None),
            ["secret", "set", "CLOUDFLARE_API_TOKEN"]
        );
    }
}
