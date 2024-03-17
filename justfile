set shell := ["powershell.exe", "-c"]

publish:
    cargo publish --package onechatsocial-config
    cargo publish --package onechatsocial-result
    cargo publish --package onechatsocial-permissions
    cargo publish --package onechatsocial-models
    cargo publish --package onechatsocial-presence
    cargo publish --package onechatsocial-database

patch:
    cargo release version patch --execute

minor:
    cargo release version minor --execute

major:
    cargo release version major --execute

release:
    scripts/try-tag-and-release.sh
