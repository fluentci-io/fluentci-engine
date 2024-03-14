use async_graphql::MergedObject;

use self::{
    cache::CacheQuery, devbox::DevboxQuery, devenv::DevenvQuery, directory::DirectoryQuery,
    file::FileQuery, flox::FloxQuery, git::GitQuery, http::HttpQuery, nix::NixQuery,
    pipeline::PipelineQuery, pkgx::PkgxQuery,
};

pub mod cache;
pub mod devbox;
pub mod devenv;
pub mod directory;
pub mod file;
pub mod flox;
pub mod git;
pub mod http;
pub mod nix;
pub mod objects;
pub mod pipeline;
pub mod pkgx;

#[derive(Default, MergedObject)]
pub struct Query(
    PipelineQuery,
    CacheQuery,
    HttpQuery,
    DirectoryQuery,
    FileQuery,
    GitQuery,
    DevboxQuery,
    DevenvQuery,
    FloxQuery,
    NixQuery,
    PkgxQuery,
);
