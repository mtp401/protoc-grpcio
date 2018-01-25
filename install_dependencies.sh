#!/usr/bin/env bash

function install_protobuf {
    local install_prefix="${INSTALL_PREFIX:-/home/travis}"
    local protobuf_version="${PROTOBUF_VERSION:-3.5.1}"
    local protobuf_dir="protobuf-${protobuf_version}"
    local protobuf_archive_name="protobuf-cpp-${protobuf_version}.tar.gz"
    local protobuf_url="https://github.com/google/protobuf/releases/download/v${protobuf_version}/${protobuf_archive_name}"
    local build_dir="/tmp"

    set -e

    if [ ! -e "${install_prefix}/bin/protoc" ]; then
        if [ ! -e "${build_dir}/${protobuf_archive_name}" ]; then
            mkdir -p "${build_dir}/download"
            wget -q -P "${build_dir}/download" "${protobuf_url}"
            mv "${build_dir}/download/${protobuf_archive_name}" "${build_dir}"
        fi
        tar -C "${build_dir}" -zxvf "${build_dir}/${protobuf_archive_name}"
        pushd "${build_dir}/${protobuf_dir}"
        ./configure --prefix="${install_prefix}"
        make
        make install
        popd
    fi

    set +e
}

install_protobuf
