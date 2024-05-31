module.exports = {
  branches: ['main'],
  plugins: [
    ["@semantic-release/exec", {
      "publishCmd": "cargo set-version ${nextRelease.version}"
    }],
    '@semantic-release/commit-analyzer',
    '@semantic-release/release-notes-generator',
    '@semantic-release/changelog',
    {
      path: '@semantic-release/git',
      assets: ['Cargo.toml', 'CHANGELOG.md'],
      message: 'chore(release): ${nextRelease.version} [skip ci]\n\n${nextRelease.notes}',
    },
    '@semantic-release/github',
    [
      "semantic-release-cargo",
      {
        "allFeatures": true,
        "check": true,
      }
    ]
  ],
};