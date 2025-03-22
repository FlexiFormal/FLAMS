#![allow(rustdoc::private_intra_doc_links)]
#![allow(unused_imports)]
/*!
 *
 * # Public API Endpoints
 *
 * | Path         | Arguments | Description / Return Value |
 * | ------------ | --------- | ----------- |
 * | **API** | <!--<hr>--> | POST requests with arguments being `application/x-www-form-urlencoded`-encoded  |
 * | [`/api/index`](backend::index()) | (None) |  |
 * | [`/api/settings`](server_fns::settings) | (None) | [Settings](SettingsSpec) (requires admin login) |
 * | [`/api/reload`](server_fns::reload) | (None) | (requires admin login) |
 * | [`/api/login`](login) | `username=<STRING>`, `password=<STRING>` | log in |
 * | [`/api/login_state`](login_state) | (None) | [LoginState] |
 * | [`/api/search`](server_fns::search) | `query=<STRING>&opts=`[`QueryFilter`](flams_ontology::search::QueryFilter)`&num_results=<INT>` | `Vec<(<FLOAT>,`[`SearchResult`](flams_ontology::search::SearchResult)`)>` |
 * | [`/api/search_symbols`](server_fns::search_symbols) | `query=<STRING>&num_results=<INT>` | `Vec<(`[`SymbolURI`]`Vec<(<FLOAT>,`[`SearchResult`](flams_ontology::search::SearchResult)`)>)>` |
 * | `/gitlab_login` |  |  |
 * | **Backend** | | |
 * | [`/api/backend/group_entries`](backend::group_entries) | (optional) `in=<STRING>` | `(Vec<`[ArchiveGroupData](crate::router::backend::ArchiveGroupData)`>,Vec<`[ArchiveData](crate::router::backend::ArchiveData)`>)` - the archives and archive groups in the provided archive group (if given) or on the top-level (if None) |
 * | [`/api/backend/archive_entries`](backend::archive_entries) | `archive=<STRING>`, (optional) `path=<STRING>` | `(Vec<`[DirectoryData](crate::router::backend::DirectoryData)`>,Vec<`[FileData](crate::router::backend::FileData)`>)` - the source directories and files in the provided archive, or (if given) the relative path within the provided archive |
 * | [`/api/backend/build_status`](backend::build_status) | `archive=<STRING>`, (optional) `path=<STRING>` | [FileStates](crate::router::backend::FileStates) - the build status of the provided archive, or (if given) the relative path within the provided archive (requires admin login) |
 * | [`/api/backend/query`](query_api) | `query=<STRING>` | `STRING` - SPARQL query endpoint; returns SPARQL JSON |
 * | [`/api/backend/archive_dependencies`](query_api) | `archives=Vec<STRING>` | `Vec<`[`ArchiveId`]`>` |
 * | **Build Queue** | | |
 * | [`/api/buildqueue/enqueue`](buildqueue::enqueue) | `archive=<STRING>`,  `target=<`[FormatOrTarget](crate::router::backend::FormatOrTarget)`>`, (optional) `path=STRING`, (optional) `stale_only=<BOOL>` (default:true) | `usize` - enqueue a new build job. Returns number of jobs queued (requires admin login)|
 * | [`/api/buildqueue/get_queues`](buildqueue::get_queues) |  | `Vec<(NonZeroU32,String)>` - return the list of all (waiting or running) build queues as (id,name) pairs (requires admin login)|
 * | [`/api/buildqueue/run`](buildqueue::run) | `id=<NonZeroU32>` | runs the build queue with the given id (requires admin login)|
 * | [`/api/buildqueue/requeue`](buildqueue::requeue) | `id=<NonZeroU32>` | requeues failed tasks in the queue with the given id (requires admin login)|
 * | [`/api/buildqueue/log`](buildqueue::get_log) | `archive=<STRING>`, `rel_path=<STRING>`, `target=<STRING>` | returns the log of the stated build job (requires admin login)|
 * | [`/api/buildqueue/migrate`](buildqueue::migrate) | `id=<NonZeroU32>` | |
 * | [`/api/buildqueue/delete`](buildqueue::delete) | `id=<NonZeroU32>` | |
 * | **Git** | | |
 * | [`/api/gitlab/get_archives`](git::get_archives) |  |  - returns the list of GitLab projects |
 * | [`/api/gitlab/get_branches`](git::get_branches) | `id=<u64>` | `Vec<`[`Branch`](flams_git::Branch)`>` - returns the list of branches for the given GitLab project |
 * | [`/api/gitlab/get_new_commits`](git::get_new_commit) | `queue=<u64>&id=ArchiveId` | `Vec<`(String,`[`Commit`](flams_git::Commit)`)`>` |
 * | **Web Sockets** | | |
 * | [`/ws/log`](crate::router::logging::LogSocket) |  |  |
 * | [`/ws/queue`](crate::router::buildqueue::QueueSocket) |  |  |
 * | **Content** | | |
 * | | | GET requests with arguments being url-encoded <br><hr> The following endpoints take a selection of arguments representing a [URI] via the following encoding: Either `uri=<STRING>` ( a full URI), or `a=<STRING>&rp=<STRING>` (an [ArchiveId] and a relative path to a source file in the archive, including file extension; only applicable for [DocumentURI]s), or  the URI components with relevant argument names; e.g. for a [DocumentURI]: `?a=<STRING>[&p=<STRING>]&l=<LANGUAGE>&d=<NAME>` |
 * | [`/img`](img_handler) | `kpse=<STRING>` or `file=<STRING>` (LSP only) or `a=<ArchiveID>&rp=<STRING>` | Images |
 * | [`/content/document`](content::document) | [DocumentURI] | `(`[DocumentURI],`Vec<`[CSS]`>,String)` Returns a pair of CSS rules and the full body of the HTML for the given document (with the `<body>` node replaced by a `<div>`, but preserving all attributes/classes) |
 * | [`/content/fragment`](content::fragment) | [URI] | `(Vec<`[CSS]`>,String)` Returns a pair of CSS rules and the HTML fragment representing the given element; i.e. the inner HTML of a document (for inputrefs), the HTML or a semantic paragraph, etc. |
 * | [`/content/omdoc`](content::omdoc) | [URI] | [`AnySpec`] Returns the structural representation of the OMDoc conent at the given URI |
 * | [`/content/toc`](content::toc()) | [DocumentURI] | `(Vec<`[CSS]`>,Vec<`[TOCElem]`>)` Returns a pair of CSS rules and the table of contents of the given document, including section titles |
 * | [`/content/los`](content::los()) | [SymbolURI] | `(Vec<(`[DocumentElementURI]`,`[LOKind]`)>` Returns a list of all Learning Objects for the given symbol |
 * | [`/content/notations`](content::notations()) | [SymbolURI] | `(Vec<(`[DocumentElementURI]`,`[Notation]`)>` Returns a list of all Notations for the given symbol or variable |
 * | [`/content/solution`](content::solution()) | [DocumentElementURI] | [`Solutions`](flams_ontology::narration::exercises::Solutions) |
 * | [`/content/quiz`](content::get_quiz()) | [DocumentURI] | [`Quiz`](flams_ontology::narration::exercises::Quiz) |
 * | [`/content/slides`](content::slides_view()) | [DocumentURI] or [DocumentElementURI] | `(Vec<CSS>,Vec<SlideElement>)` |
 * | [`/content/legacy/uris`](content::uris()) | | |
*/

use crate::server::img::img_handler;

use flams_ontology::{
    narration::{notations::Notation, LOKind},
    uris::*,
};
use flams_router_dashboard::{
    server_fns::{
        self, backend, buildqueue, content, git,
        login::{login, login_state},
    },
    LoginState,
};
use flams_utils::{settings::SettingsSpec, CSS};
use ftml_viewer_components::components::{omdoc::AnySpec, TOCElem};
