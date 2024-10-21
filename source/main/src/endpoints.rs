/*! # Public API Endpoints
 * 
 * | Path         | Arguments | Description / Return Value |
 * | ------------ | --------- | ----------- |
 * | **API** | <!--<hr>--> | POST requests with arguments being `application/x-www-form-urlencoded`-encoded  |
 * | [`/api/settings`](crate::router::settings::get_settings) | (None) | [Settings](immt_utils::settings::SettingsSpec) (requires admin login) |
 * | [`/api/login`](crate::users::login) | `username=<STRING>`, `password=<STRING>` | log in |
 * | [`/api/login_state`](crate::users::login_state) | (None) | [LoginState](crate::users::LoginState) |
 * | **Backend** | | |
 * | [`/api/backend/group_entries`](crate::router::backend::group_entries) | (optional) `in=<STRING>` | `(Vec<`[ArchiveGroupData](crate::router::backend::ArchiveGroupData)`>,Vec<`[ArchiveData](crate::router::backend::ArchiveData)`>)` - the archives and archive groups in the provided archive group (if given) or on the top-level (if None) |
 * | [`/api/backend/archive_entries`](crate::router::backend::archive_entries) | `archive=<STRING>`, (optional) `path=<STRING>` | `(Vec<`[DirectoryData](crate::router::backend::DirectoryData)`>,Vec<`[FileData](crate::router::backend::FileData)`>)` - the source directories and files in the provided archive, or (if given) the relative path within the provided archive |
 * | [`/api/backend/build_status`](crate::router::backend::build_status) | `archive=<STRING>`, (optional) `path=<STRING>` | [FileStates](crate::router::backend::FileStates) - the build status of the provided archive, or (if given) the relative path within the provided archive (requires admin login) |
 * | [`/api/backend/query`](crate::router::query::query_api) | `query=<STRING>` | `STRING` - SPARQL query endpoint; returns SPARQL JSON |
 * | **Build Queue** | | |
 * | [`/api/buldqueue/enqueue`](crate::router::backend::enqueue) | `archive=<STRING>`,  `target=<`[FormatOrTarget](crate::router::backend::FormatOrTarget)`>`, (optional) `path=STRING`, (optional) `stale_only=<BOOL>` (default:true) | `usize` - enqueue a new build job. Returns number of jobs queued (requires admin login)|
 * | [`/api/buldqueue/get_queues`](crate::router::buildqueue::get_queues) |  | `Vec<(NonZeroU32,String)>` - return the list of all (waiting or running) build queues as (id,name) pairs (requires admin login)|
 * | [`/api/buldqueue/run`](crate::router::buildqueue::run) | `id=<NonZeroU32>` | runs the build queue with the given id (requires admin login)|
 * | [`/api/buldqueue/log`](crate::router::buildqueue::get_log) | `archive=<STRING>`, `rel_path=<STRING>`, `target=<STRING>` | returns the log of the stated build job (requires admin login)|
 * | **Web Sockets** | | |
 * | [`/ws/log`](crate::router::logging::LogSocket) |  |  |
 * | [`/ws/queue`](crate::router::buildqueue::QueueSocket) |  |  |
 * | **Content** | | |
 * | | | GET requests with arguments being url-encoded <br><hr> The following endpoints take a selection of arguments representing a [URI](immt_ontology::uris) via the following encoding: Either `uri=<STRING>` ( a full URI), or `a=<STRING>&rp=<STRING>` (an [ArchiveId](immt_ontology::uris::ArchiveId) and a relative path to a source file in the archive, including file extension; only applicable for [DocumentURIs](immt_ontology::uris::DocumentURI)), or  the URI components with relevant argument names; e.g. for a [DocumentURI](immt_ontology::uris::DocumentURI): `?a=<STRING>[&p=<STRING>]&l=<LANGUAGE>&d=<NAME>` |
 * | [`/content/document`](crate::router::content::document) | [DocumentURI](immt_ontology::uris::DocumentURI) | `(Vec<`[CSS](immt_utils::CSS)`>,String)` Returns a pair of CSS rules and the full body of the HTML for the given document (with the `<body>` node replaced by a `<div>`, but preserving all attributes/classes) |
 * | [`/content/fragment`](crate::router::content::fragment) | [URI](immt_ontology::uris::URI) | `(Vec<`[CSS](immt_utils::CSS)`>,String)` Returns a pair of CSS rules and the HTML fragment representing the given element; i.e. the inner HTML of a document (for inputrefs), the HTML or a semantic paragraph, etc. |
 * | [`/content/toc`](crate::router::content::toc()) | [DocumentURI](immt_ontology::uris::DocumentURI) | `(Vec<`[CSS](immt_utils::CSS)`>,Vec<`[TOCElem](shtml_viewer_components::components::TOCElem)`>)` Returns a pair of CSS rules and the table of contents of the given document, including section titles |
*/