#!/usr/bin/env bash
# Building and packaging for release

set -ex

build() {
    cargo build --target "${TARGET}" --release --verbose
}

create_package() {
    local tempdir
    local out_dir
    local PROJECT_NAME
    local package_name
    local deploy_dir

    PROJECT_NAME="rnr"
    tempdir=$(mktemp -d 2>/dev/null || mktemp -d -t tmp)
    out_dir=$(pwd)
    package_name="${PROJECT_NAME}-${TRAVIS_TAG}-${TARGET}"
    deploy_dir="${tempdir}/${package_name}"

    # create a deployment directory
    mkdir "${deploy_dir}"
    mkdir "${deploy_dir}/completion"

    # copy files
    cp "target/${TARGET}/release/${PROJECT_NAME}" "${deploy_dir}/"
    cp "target/${TARGET}/release/build/rnr"-*/out/"_rnr" "${deploy_dir}/completion/"
    cp "target/${TARGET}/release/build/rnr"-*/out/"rnr.bash" "${deploy_dir}/completion/"
    cp "target/${TARGET}/release/build/rnr"-*/out/"rnr.fish" "${deploy_dir}/completion/"
    cp "target/${TARGET}/release/build/rnr"-*/out/"_rnr.ps1" "${deploy_dir}/completion/"
    cp README.md "${deploy_dir}"
    cp LICENSE "${deploy_dir}"

    # archiving
    pushd "${tempdir}"
    tar czf "${out_dir}/${package_name}.tar.gz" "${package_name}"/*
    popd
    rm -r "${tempdir}"
}

main() {
    build
    create_package
}

main
