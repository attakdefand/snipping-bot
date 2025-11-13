//! GitHub release signal detection for social events
use serde::{Deserialize, Serialize};
use sniper_core::types::{ChainRef, Signal};
use tracing::{debug, info, warn};

/// GitHub release data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubRelease {
    /// Repository owner
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// Release tag
    pub tag: String,
    /// Release name
    pub name: String,
    /// Release description/body
    pub body: String,
    /// Release URL
    pub url: String,
    /// Whether this is a pre-release
    pub prerelease: bool,
    /// Whether this is a draft
    pub draft: bool,
    /// Timestamp of the release
    pub timestamp: u64,
    /// Author of the release
    pub author: String,
}

/// GitHub release signal detector
pub struct GithubReleaseDetector {
    /// Chain this detector is monitoring
    chain: ChainRef,
    /// List of watched repositories
    watched_repos: Vec<(String, String)>, // (owner, repo)
}

impl GithubReleaseDetector {
    /// Create a new GitHub release detector
    pub fn new(chain: ChainRef, watched_repos: Vec<(String, String)>) -> Self {
        Self {
            chain,
            watched_repos,
        }
    }

    /// Process a GitHub release and generate a signal if it's from a watched repo
    pub fn process_github_release(&self, release: GithubRelease) -> Option<Signal> {
        info!(
            "Processing GitHub release {} for {}/{} on chain {}",
            release.tag, release.owner, release.repo, self.chain.name
        );

        // Check if this is from a watched repository
        let is_watched = self
            .watched_repos
            .iter()
            .any(|(owner, repo)| owner == &release.owner && repo == &release.repo);

        if !is_watched {
            debug!("Release is not from a watched repository, ignoring");
            return None;
        }

        // Skip draft releases
        if release.draft {
            debug!("Release is a draft, ignoring");
            return None;
        }

        // Create the signal
        let signal = Signal {
            source: "social".to_string(),
            kind: if release.prerelease {
                "github_prerelease".to_string()
            } else {
                "github_release".to_string()
            },
            chain: self.chain.clone(),
            token0: None,
            token1: None,
            extra: serde_json::json!({
                "owner": release.owner,
                "repo": release.repo,
                "tag": release.tag,
                "name": release.name,
                "body": release.body,
                "url": release.url,
                "prerelease": release.prerelease,
                "draft": release.draft,
                "timestamp": release.timestamp,
                "author": release.author,
            }),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };

        debug!("Generated GitHub release signal: {:?}", signal);
        Some(signal)
    }

    /// Validate a GitHub release
    pub fn validate_github_release(&self, release: &GithubRelease) -> bool {
        // Basic validation
        if release.owner.is_empty() {
            warn!("Invalid owner in GitHub release");
            return false;
        }

        if release.repo.is_empty() {
            warn!("Invalid repo in GitHub release");
            return false;
        }

        if release.tag.is_empty() {
            warn!("Invalid tag in GitHub release");
            return false;
        }

        if release.url.is_empty() {
            warn!("Invalid URL in GitHub release");
            return false;
        }

        if release.timestamp == 0 {
            warn!("Invalid timestamp in GitHub release");
            return false;
        }

        if release.author.is_empty() {
            warn!("Invalid author in GitHub release");
            return false;
        }

        true
    }

    /// Filter GitHub releases based on criteria
    pub fn filter_github_release(&self, release: &GithubRelease) -> bool {
        // In a real implementation, this would check against configured filters
        // For now, we'll accept all valid releases from watched repos
        self.validate_github_release(release)
    }

    /// Add a repository to the watch list
    pub fn add_watched_repo(&mut self, owner: String, repo: String) {
        self.watched_repos.push((owner, repo));
    }

    /// Remove a repository from the watch list
    pub fn remove_watched_repo(&mut self, owner: &str, repo: &str) {
        self.watched_repos.retain(|(o, r)| o != owner || r != repo);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::ChainRef;

    #[test]
    fn test_github_release_detector_creation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let watched_repos = vec![("owner1".to_string(), "repo1".to_string())];
        let detector = GithubReleaseDetector::new(chain.clone(), watched_repos.clone());
        assert_eq!(detector.chain.name, "ethereum");
        assert_eq!(detector.chain.id, 1);
        assert_eq!(detector.watched_repos, watched_repos);
    }

    #[test]
    fn test_github_release_validation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector =
            GithubReleaseDetector::new(chain, vec![("owner1".to_string(), "repo1".to_string())]);

        // Valid release
        let valid_release = GithubRelease {
            owner: "owner1".to_string(),
            repo: "repo1".to_string(),
            tag: "v1.0.0".to_string(),
            name: "Version 1.0.0".to_string(),
            body: "Release notes".to_string(),
            url: "https://github.com/owner1/repo1/releases/tag/v1.0.0".to_string(),
            prerelease: false,
            draft: false,
            timestamp: 1234567890,
            author: "author1".to_string(),
        };

        assert!(detector.validate_github_release(&valid_release));

        // Invalid release - empty owner
        let mut invalid_release = valid_release.clone();
        invalid_release.owner = String::new();
        assert!(!detector.validate_github_release(&invalid_release));
    }

    #[test]
    fn test_github_release_signal_generation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector =
            GithubReleaseDetector::new(chain, vec![("owner1".to_string(), "repo1".to_string())]);

        // Release from watched repo
        let watched_release = GithubRelease {
            owner: "owner1".to_string(),
            repo: "repo1".to_string(),
            tag: "v1.0.0".to_string(),
            name: "Version 1.0.0".to_string(),
            body: "Release notes".to_string(),
            url: "https://github.com/owner1/repo1/releases/tag/v1.0.0".to_string(),
            prerelease: false,
            draft: false,
            timestamp: 1234567890,
            author: "author1".to_string(),
        };

        let signal = detector.process_github_release(watched_release);
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.source, "social");
        assert_eq!(signal.kind, "github_release");
        assert_eq!(signal.chain.name, "ethereum");
        assert!(signal.seen_at_ms > 0);

        // Release from unwatched repo
        let unwatched_release = GithubRelease {
            owner: "owner2".to_string(),
            repo: "repo2".to_string(),
            tag: "v1.0.0".to_string(),
            name: "Version 1.0.0".to_string(),
            body: "Release notes".to_string(),
            url: "https://github.com/owner2/repo2/releases/tag/v1.0.0".to_string(),
            prerelease: false,
            draft: false,
            timestamp: 1234567890,
            author: "author2".to_string(),
        };

        let signal = detector.process_github_release(unwatched_release);
        assert!(signal.is_none());

        // Pre-release from watched repo
        let prerelease = GithubRelease {
            owner: "owner1".to_string(),
            repo: "repo1".to_string(),
            tag: "v1.0.0-beta".to_string(),
            name: "Version 1.0.0 Beta".to_string(),
            body: "Pre-release notes".to_string(),
            url: "https://github.com/owner1/repo1/releases/tag/v1.0.0-beta".to_string(),
            prerelease: true,
            draft: false,
            timestamp: 1234567890,
            author: "author1".to_string(),
        };

        let signal = detector.process_github_release(prerelease);
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.kind, "github_prerelease");

        // Draft release from watched repo (should be ignored)
        let draft_release = GithubRelease {
            owner: "owner1".to_string(),
            repo: "repo1".to_string(),
            tag: "v1.0.0-draft".to_string(),
            name: "Version 1.0.0 Draft".to_string(),
            body: "Draft notes".to_string(),
            url: "https://github.com/owner1/repo1/releases/tag/v1.0.0-draft".to_string(),
            prerelease: false,
            draft: true,
            timestamp: 1234567890,
            author: "author1".to_string(),
        };

        let signal = detector.process_github_release(draft_release);
        assert!(signal.is_none());
    }

    #[test]
    fn test_watched_repo_management() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let mut detector = GithubReleaseDetector::new(chain, vec![]);

        // Add a repo
        detector.add_watched_repo("owner1".to_string(), "repo1".to_string());
        assert_eq!(detector.watched_repos.len(), 1);
        assert_eq!(
            detector.watched_repos[0],
            ("owner1".to_string(), "repo1".to_string())
        );

        // Remove a repo
        detector.remove_watched_repo("owner1", "repo1");
        assert_eq!(detector.watched_repos.len(), 0);
    }
}
