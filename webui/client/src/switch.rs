use yew_router::prelude::*;

#[derive(PartialEq, Clone, Debug, Routable)]
pub enum Route {
    #[at("/posts/:id")]
    Post { id: u64 },
    #[at("/posts/?page=:id")]
    PostListPage { id: u64 },
    #[at("/posts/")]
    PostList,
    #[at("/authors/")]
    VersionInfo,
    //#[not_found]
    #[at("/")]
    Home,
}
