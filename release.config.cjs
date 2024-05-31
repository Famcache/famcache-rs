module.exports = {
  branches: ['main'],
  plugins: [
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
        "checkArgs": ["--no-deps"],
        "publishArgs": ["--no-verify"]
      }
    ]
  ],
};