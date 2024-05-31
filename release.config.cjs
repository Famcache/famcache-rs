module.exports = {
  branches: ['main'],
  plugins: [
    ["@semantic-release/exec", {
      "publishCmd": "cargo set-version ${nextRelease.version}"
    }],
    '@semantic-release/commit-analyzer',
    '@semantic-release/changelog',
    '@semantic-release/release-notes-generator',
    [
      "semantic-release-cargo",
      {
        "allFeatures": true,
        "check": true,
      }
    ],
    ["@semantic-release/git", {
      "assets": ["CHANGELOG.md", "Cargo.toml", "Cargo.lock"],
      "message": "chore(release): ${nextRelease.version} [skip ci]\n\n${nextRelease.notes}"
    }],
    '@semantic-release/github',
  ],
};