//! The module with the [`Project`] presenter. See [`crate::presenter`] documentation to know more
//! about presenters in general.

use crate::prelude::*;

use crate::executor::global::spawn_stream_handler;
use crate::presenter;
use crate::presenter::graph::ViewNodeId;

use engine_protocol::language_server::ExecutionEnvironment;
use enso_frp as frp;
use ensogl::system::js;
use ide_view as view;
use ide_view::project::SearcherParams;
use model::module::NotificationKind;
use model::project::Notification;
use model::project::VcsStatus;



// =================
// === Constants ===
// =================

/// We don't know how long the project opening will take, but we still want to show a fake progress
/// indicator for the user. This constant represents a progress percentage that will be displayed.
const OPEN_PROJECT_SPINNER_PROGRESS: f32 = 0.8;



// =============
// === Model ===
// =============

// Those fields will be probably used when Searcher and Breadcrumbs integration will be implemented.
#[allow(unused)]
#[derive(Debug)]
struct Model {
    controller:         controller::Project,
    module_model:       model::Module,
    graph_controller:   controller::ExecutedGraph,
    ide_controller:     controller::Ide,
    view:               view::project::View,
    status_bar:         view::status_bar::View,
    graph:              presenter::Graph,
    code:               presenter::Code,
    searcher:           RefCell<Option<presenter::Searcher>>,
    available_projects: Rc<RefCell<Vec<(ImString, Uuid)>>>,
}

impl Model {
    #[profile(Task)]
    fn new(
        ide_controller: controller::Ide,
        controller: controller::Project,
        init_result: controller::project::InitializationResult,
        view: view::project::View,
        status_bar: view::status_bar::View,
    ) -> Self {
        let graph_controller = init_result.main_graph;
        let text_controller = init_result.main_module_text;
        let module_model = init_result.main_module_model;
        let graph = presenter::Graph::new(
            controller.model.clone_ref(),
            graph_controller.clone_ref(),
            &view,
        );
        let code = presenter::Code::new(text_controller, &view);
        let searcher = default();
        let available_projects = default();
        Model {
            controller,
            module_model,
            graph_controller,
            ide_controller,
            view,
            status_bar,
            graph,
            code,
            searcher,
            available_projects,
        }
    }

    fn setup_searcher_presenter(&self, params: SearcherParams) {
        let new_presenter = presenter::Searcher::setup_controller(
            self.ide_controller.clone_ref(),
            self.controller.clone_ref(),
            self.graph_controller.clone_ref(),
            &self.graph,
            self.view.clone_ref(),
            params,
        );
        match new_presenter {
            Ok(searcher) => {
                *self.searcher.borrow_mut() = Some(searcher);
            }
            Err(err) => {
                error!("Error while creating searcher integration: {err}");
            }
        }
    }

    fn editing_committed_old_searcher(
        &self,
        node: ViewNodeId,
        entry_id: Option<view::searcher::entry::Id>,
    ) -> bool {
        let searcher = self.searcher.take();
        if let Some(searcher) = searcher {
            let is_example = entry_id.map_or(false, |i| searcher.is_entry_an_example(i));
            if let Some(created_node) = searcher.commit_editing(entry_id) {
                self.graph.allow_expression_auto_updates(created_node, true);
                if is_example {
                    self.view.graph().enable_visualization(node);
                }
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    fn editing_committed(
        &self,
        entry_id: Option<view::component_browser::component_list_panel::grid::GroupEntryId>,
    ) -> bool {
        let searcher = self.searcher.take();
        if let Some(searcher) = searcher {
            if let Some(created_node) = searcher.expression_accepted(entry_id) {
                self.graph.allow_expression_auto_updates(created_node, true);
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    fn editing_aborted(&self) {
        let searcher = self.searcher.take();
        if let Some(searcher) = searcher {
            let input_node_view = searcher.input_view();
            if let Some(node) = self.graph.ast_node_of_view(input_node_view) {
                self.graph.allow_expression_auto_updates(node, true);
                searcher.abort_editing();
            } else {
                warn!("When porting editing the AST node of the node view {input_node_view} could not be found.");
            }
        } else {
            warn!("Editing aborted without searcher controller.");
        }
    }

    fn rename_project(&self, name: impl Str) {
        if self.controller.model.name() != name.as_ref() {
            let project = self.controller.model.clone_ref();
            let breadcrumbs = self.view.graph().model.breadcrumbs.clone_ref();
            let name = name.into();
            executor::global::spawn(async move {
                if let Err(e) = project.rename_project(name).await {
                    error!("The project couldn't be renamed: {e}");
                    breadcrumbs.cancel_project_name_editing.emit(());
                }
            });
        }
    }

    fn undo(&self) {
        debug!("Undo triggered in UI.");
        if let Err(e) = self.controller.model.urm().undo() {
            error!("Undo failed: {e}");
        }
    }

    fn redo(&self) {
        debug!("Redo triggered in UI.");
        if let Err(e) = self.controller.model.urm().redo() {
            error!("Redo failed: {e}");
        }
    }

    fn save_project_snapshot(&self) {
        let controller = self.controller.clone_ref();
        let breadcrumbs = self.view.graph().model.breadcrumbs.clone_ref();
        executor::global::spawn(async move {
            if let Err(err) = controller.save_project_snapshot().await {
                error!("Error while saving project snapshot: {err}");
            } else {
                breadcrumbs.set_project_changed(false);
            }
        })
    }

    fn toggle_component_browser_private_entries_visibility(&self) {
        let visibility = self.ide_controller.are_component_browser_private_entries_visible();
        self.ide_controller.set_component_browser_private_entries_visibility(!visibility);
    }

    /// Toggle the read-only mode, return the new state.
    fn toggle_read_only(&self) -> bool {
        let current_state = self.controller.model.read_only();
        let new_state = !current_state;
        self.set_read_only(new_state);
        new_state
    }
    /// Toggle the read-only mode, return the new state.

    fn set_read_only(&self, new_state: bool) {
        self.controller.model.set_read_only(new_state);
        info!("New read only state: {}.", new_state);
    }

    fn restore_project_snapshot(&self) {
        let controller = self.controller.clone_ref();
        let breadcrumbs = self.view.graph().model.breadcrumbs.clone_ref();
        executor::global::spawn(async move {
            if let Err(err) = controller.restore_project_snapshot().await {
                error!("Error while restoring project snapshot: {err}");
            } else {
                breadcrumbs.set_project_changed(false);
            }
        })
    }

    fn set_project_changed(&self, changed: bool) {
        self.view.graph().model.breadcrumbs.set_project_changed(changed);
    }

    fn execution_context_interrupt(&self) {
        let controller = self.graph_controller.clone_ref();
        executor::global::spawn(async move {
            if let Err(err) = controller.interrupt().await {
                error!("Error interrupting execution context: {err}");
            }
        })
    }

    fn execution_context_restart(&self) {
        let controller = self.graph_controller.clone_ref();
        executor::global::spawn(async move {
            if let Err(err) = controller.restart().await {
                error!("Error restarting execution context: {err}");
            }
        })
    }

    /// Prepare a list of projects to display in the Open Project dialog.
    fn project_list_opened(&self, project_list_ready: frp::Source<()>) {
        let controller = self.ide_controller.clone_ref();
        let projects_list = self.available_projects.clone_ref();
        executor::global::spawn(async move {
            if let Ok(api) = controller.manage_projects() {
                if let Ok(projects) = api.list_projects().await {
                    let projects = projects.into_iter();
                    let projects = projects.map(|p| (p.name.clone().into(), p.id)).collect_vec();
                    *projects_list.borrow_mut() = projects;
                    project_list_ready.emit(());
                }
            }
        })
    }

    /// User clicked a project in the Open Project dialog. Open it.
    fn open_project(&self, id_in_list: &usize) {
        let controller = self.ide_controller.clone_ref();
        let projects_list = self.available_projects.clone_ref();
        let view = self.view.clone_ref();
        let status_bar = self.status_bar.clone_ref();
        let id = *id_in_list;
        executor::global::spawn(async move {
            let app = js::app_or_panic();
            app.show_progress_indicator(OPEN_PROJECT_SPINNER_PROGRESS);
            view.hide_graph_editor();
            if let Ok(api) = controller.manage_projects() {
                api.close_project();
                let uuid = projects_list.borrow().get(id).map(|(_name, uuid)| *uuid);
                if let Some(uuid) = uuid {
                    if let Err(error) = api.open_project(uuid).await {
                        error!("Error opening project: {error}.");
                        status_bar.add_event(format!("Error opening project: {error}."));
                    }
                } else {
                    error!("Project with id {id} not found.");
                }
            } else {
                error!("Project Manager API not available, cannot open project.");
            }
            app.hide_progress_indicator();
            view.show_graph_editor();
        })
    }

    fn execution_environment_changed(
        &self,
        execution_environment: &ide_view::execution_environment_selector::ExecutionEnvironment,
    ) {
        if let Ok(execution_environment) = execution_environment.as_str().try_into() {
            let graph_controller = self.graph_controller.clone_ref();
            executor::global::spawn(async move {
                if let Err(err) =
                    graph_controller.set_execution_environment(execution_environment).await
                {
                    error!("Error setting execution environment: {err}");
                }
            });
        } else {
            error!("Invalid execution environment: {execution_environment:?}");
        }
    }
}


// ===============
// === Project ===
// ===============

/// The Project Presenter, synchronizing state between project controller and project view.
#[derive(Clone, CloneRef, Debug)]
pub struct Project {
    network: frp::Network,
    model:   Rc<Model>,
}

impl Project {
    /// Construct new project presenter, basing of the project initialization result.
    ///
    /// The returned presenter will be already working: it will display the initial main graph, and
    /// react to all notifications.
    #[profile(Task)]
    pub fn new(
        ide_controller: controller::Ide,
        controller: controller::Project,
        init_result: controller::project::InitializationResult,
        view: view::project::View,
        status_bar: view::status_bar::View,
    ) -> Self {
        let network = frp::Network::new("presenter::Project");
        let model = Model::new(ide_controller, controller, init_result, view, status_bar);
        let model = Rc::new(model);
        Self { network, model }.init()
    }

    #[profile(Detail)]
    fn init(self) -> Self {
        let model = &self.model;
        let network = &self.network;

        let view = &model.view.frp;
        let breadcrumbs = &model.view.graph().model.breadcrumbs;
        let graph_view = &model.view.graph().frp;
        let project_list = &model.view.project_list();

        frp::extend! { network
            project_list_ready <- source_();

            project_list.grid.reset_entries <+ project_list_ready.map(f_!([model]{
                let cols = 1;
                let rows = model.available_projects.borrow().len();
                (rows, cols)
            }));
            entry_model <- project_list.grid.model_for_entry_needed.map(f!([model]((row, col)) {
                let projects = model.available_projects.borrow();
                let project = projects.get(*row);
                project.map(|(name, _)| (*row, *col, name.clone_ref()))
            })).filter_map(|t| t.clone());
            project_list.grid.model_for_entry <+ entry_model;

            open_project_list <- view.project_list_shown.on_true();
            eval_ open_project_list(model.project_list_opened(project_list_ready.clone_ref()));
            selected_project <- project_list.grid.entry_selected.filter_map(|e| *e);
            eval selected_project(((row, _col)) model.open_project(row));
            project_list.grid.select_entry <+ selected_project.constant(None);

            eval view.searcher ([model](params) {
                if let Some(params) = params {
                    model.setup_searcher_presenter(*params)
                }
            });

            graph_view.remove_node <+ view.editing_committed_old_searcher.filter_map(f!([model]((node_view, entry)) {
                model.editing_committed_old_searcher(*node_view, *entry).as_some(*node_view)
            }));
            graph_view.remove_node <+ view.editing_committed.filter_map(f!([model]((node_view, entry)) {
                model.editing_committed(*entry).as_some(*node_view)
            }));
            eval_ view.editing_aborted(model.editing_aborted());

            eval breadcrumbs.output.project_name((name) {model.rename_project(name);});

            eval_ view.undo (model.undo());
            eval_ view.redo (model.redo());

            values_computed <- source::<()>();
            view.values_updated <+ values_computed;

            eval_ view.save_project_snapshot(model.save_project_snapshot());
            eval_ view.restore_project_snapshot(model.restore_project_snapshot());

            eval_ view.toggle_component_browser_private_entries_visibility(
                model.toggle_component_browser_private_entries_visibility()
            );

            eval_ view.execution_context_interrupt(model.execution_context_interrupt());

            eval_ view.execution_context_restart(model.execution_context_restart());

            view.set_read_only <+ view.toggle_read_only.map(f_!(model.toggle_read_only()));
            eval graph_view.execution_environment((env) model.execution_environment_changed(env));
            view.set_read_only <+ graph_view.execution_environment.map(f!([model](env) {
                let read_only = ExecutionEnvironment::try_from(env.as_ref()).map(|env| !env.output_context_enabled()).unwrap_or(false);
                model.set_read_only(read_only);
                read_only
            }));

        }

        let graph_controller = self.model.graph_controller.clone_ref();

        self.init_analytics()
            .init_execution_environments()
            .setup_notification_handler()
            .attach_frp_to_values_computed_notifications(graph_controller, values_computed)
    }

    /// Initialises execution environment.
    fn init_execution_environments(self) -> Self {
        let graph = &self.model.view.graph();
        let entries = Rc::new(ExecutionEnvironment::list_all_as_imstrings());
        graph.set_available_execution_environments(entries);
        self
    }

    fn init_analytics(self) -> Self {
        let network = &self.network;
        let project = &self.model.view;
        let graph = self.model.view.graph();
        frp::extend! { network
            eval_ graph.node_editing_started([]analytics::remote_log_event("graph_editor::node_editing_started"));
            eval_ graph.node_editing_finished([]analytics::remote_log_event("graph_editor::node_editing_finished"));
            eval_ graph.node_added([]analytics::remote_log_event("graph_editor::node_added"));
            eval_ graph.node_removed([]analytics::remote_log_event("graph_editor::node_removed"));
            eval_ graph.nodes_collapsed([]analytics::remote_log_event("graph_editor::nodes_collapsed"));
            eval_ graph.node_entered([]analytics::remote_log_event("graph_editor::node_enter_request"));
            eval_ graph.node_exited([]analytics::remote_log_event("graph_editor::node_exit_request"));
            eval_ graph.on_edge_endpoints_set([]analytics::remote_log_event("graph_editor::edge_endpoints_set"));
            eval_ graph.visualization_shown([]analytics::remote_log_event("graph_editor::visualization_shown"));
            eval_ graph.visualization_hidden([]analytics::remote_log_event("graph_editor::visualization_hidden"));
            eval_ graph.on_edge_endpoint_unset([]analytics::remote_log_event("graph_editor::connection_removed"));
            eval_ project.editing_committed([]analytics::remote_log_event("project::editing_committed"));
        }
        self
    }

    fn setup_notification_handler(self) -> Self {
        let notifications = self.model.controller.model.subscribe();
        let weak = Rc::downgrade(&self.model);
        spawn_stream_handler(weak, notifications, |notification, model| {
            info!("Processing notification {notification:?}");
            match notification {
                Notification::ConnectionLost(_) => {
                    let message = crate::BACKEND_DISCONNECTED_MESSAGE;
                    let message = view::status_bar::event::Label::from(message);
                    model.status_bar.add_event(message);
                }
                Notification::VcsStatusChanged(VcsStatus::Dirty) => {
                    model.set_project_changed(true);
                }
                Notification::VcsStatusChanged(VcsStatus::Clean) => {
                    model.set_project_changed(false);
                }
            };
            std::future::ready(())
        });

        let notifications = self.model.module_model.subscribe();
        let weak = Rc::downgrade(&self.model);
        spawn_stream_handler(weak, notifications, move |notification, model| {
            match notification.kind {
                NotificationKind::Invalidate
                | NotificationKind::CodeChanged { .. }
                | NotificationKind::MetadataChanged => model.set_project_changed(true),
                NotificationKind::Reloaded => model.set_project_changed(false),
            }
            futures::future::ready(())
        });
        self
    }

    fn attach_frp_to_values_computed_notifications(
        self,
        graph: controller::ExecutedGraph,
        values_computed: frp::Source<()>,
    ) -> Self {
        let weak = Rc::downgrade(&self.model);
        let notifications = graph.subscribe();
        spawn_stream_handler(weak, notifications, move |notification, _| {
            if let controller::graph::executed::Notification::ComputedValueInfo(_) = notification {
                values_computed.emit(());
            }
            std::future::ready(())
        });
        self
    }

    /// Initialize project and return working presenter.
    ///
    /// This calls the [`controller::Project::initialize`] method and use the initialization result
    /// to construct working presenter.
    #[profile(Task)]
    pub async fn initialize(
        ide_controller: controller::Ide,
        controller: controller::Project,
        view: view::project::View,
        status_bar: view::status_bar::View,
    ) -> FallibleResult<Self> {
        let init_result = controller.initialize().await?;
        Ok(Self::new(ide_controller, controller, init_result, view, status_bar))
    }
}
