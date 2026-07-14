# Release
- `git cliff --bump -o CHANGELOG.md`
- Change cargo workspace version to new cliff version
- Check `dist plan`
- `git commit -am "Chore: Update CHANGELOG.md and Release X.Y.Z"`
- `git tag "vX.Y.Z"`
- `git push`
- `git push --tags`
