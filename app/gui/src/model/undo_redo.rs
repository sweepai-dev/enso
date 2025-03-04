//! Support for IDE Undo-Redo functionality.

use crate::prelude::*;

use crate::controller;



// ==============
// === Errors ===
// ==============

#[allow(missing_docs)]
#[derive(Debug, Clone, Eq, Fail, PartialEq)]
#[fail(display = "Cannot undo because there is an ongoing transaction '{}'.", transaction_name)]
pub struct CannotUndoDuringTransaction {
    transaction_name: String,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Eq, Fail, PartialEq)]
#[fail(display = "There is no action stored that can be undone.")]
pub struct NoActionToUndo;

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Eq, Fail, PartialEq)]
#[fail(display = "The faux undo transaction was leaked.")]
pub struct FauxTransactionLeaked;

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Eq, Fail, PartialEq)]
#[fail(display = "No frame to pop, {} stack is empty.", _0)]
pub struct NoFrameToPop(Stack);

#[allow(missing_docs)]
#[derive(Debug, Clone, Eq, Fail, PartialEq)]
#[fail(display = "The module {} is not accessible.", _0)]
pub struct MissingModuleHandle(String);



// ==============
// === Traits ===
// ==============

/// Trait represents undo-aware type that is able to access undo-redo repository.
///
/// It allows to open transactions and check state of the repository.
/// It does not allow however to execute undo/redo itself, this is done through [`Manager`].
pub trait Aware {
    /// Get handle to undo-redo [`Repository`].
    fn undo_redo_repository(&self) -> Rc<Repository>;

    /// Get the current ongoing transaction. If there is no ongoing transaction, create one.
    #[profile(Debug)]
    #[must_use]
    fn get_or_open_transaction(&self, name: &str) -> Rc<Transaction> {
        self.undo_redo_repository().transaction(name)
    }
}



// ===================
// === Transaction ===
// ===================

/// Transaction is a RAII-style object used to group a number of actions into a single undoable
/// operation.
///
/// When the transaction is dropped, it adds itself to the undo stack, unless it was ignored.
#[derive(Debug)]
pub struct Transaction {
    #[allow(missing_docs)]
    frame:   RefCell<Frame>,
    urm:     Weak<Repository>,
    ignored: Cell<bool>,
}

impl Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Transaction '{}'", self.frame.borrow())
    }
}

impl Transaction {
    /// Create a new transaction, that will add to the given's repository undo stack on destruction.
    pub fn new(urm: &Rc<Repository>, name: String) -> Self {
        Self {
            frame:   RefCell::new(Frame { name, ..default() }),
            urm:     Rc::downgrade(urm),
            ignored: default(),
        }
    }

    /// Get the transaction name.
    ///
    /// Currently the name serves only debugging purposes.
    pub fn name(&self) -> String {
        self.frame.borrow().name.clone()
    }

    /// Stores the state of given module.
    ///
    /// This is the state that will be restored, when the transaction is undone. As such is should
    /// be the state "from before" the undoable action.
    ///
    /// This method stores content only once for given module. Thus it is safe to call this on
    /// the current transaction in context where it is not clear whether transaction was already set
    /// up or not.
    pub fn fill_content(&self, id: model::module::Id, content: model::module::Content) {
        with(self.frame.borrow_mut(), |mut data| data.store_module_content(id, content))
    }

    /// Ignore the transaction.
    ///
    /// Ignored transaction when dropped is discarded, rather than being put on top of "Redo" stack.
    /// It does not affect the actions belonging to transaction in any way.
    pub fn ignore(&self) {
        info!("Marking transaction '{}' as ignored.", self.frame.borrow().name);
        self.ignored.set(true)
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if let Some(urm) = self.urm.upgrade() {
            urm.close_transaction(self)
        }
    }
}



// =============
// === Frame ===
// =============

/// Frame represents a state stored on undo or redo stack.
///
/// [`Manager`] is able to restore project's state to a given `Frame`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    /// Name of the transaction that created this frame.
    pub name:            String,
    /// Names of all subsequent transactions that were covered by this frame. This serves the
    /// debugging purposes only, to understand what actions are covered by given frame.
    pub secondary_names: Vec<String>,
    /// Context module where the change was made.
    pub module:          Option<model::module::Id>,
    /// Context graph where the change was made.
    pub graph:           Option<controller::graph::Id>,
    /// Snapshots of content for all edited modules.
    pub snapshots:       BTreeMap<model::module::Id, model::module::Content>,
}

impl Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}; ", self.name)?;
        if !self.secondary_names.is_empty() {
            writeln!(f, "Secondary names:")?;
            for name in &self.secondary_names {
                writeln!(f, "\t{name}")?;
            }
        }
        if let Some(m) = &self.module {
            writeln!(f, "Module: {m}; ")?;
        }
        if let Some(g) = &self.graph {
            writeln!(f, "Graph: {g}; ")?;
        }
        for (id, code) in &self.snapshots {
            writeln!(f, "Code for {id}: {code}; ")?;
        }
        Ok(())
    }
}

impl Frame {
    /// Store the snapshot of given module's content.
    pub fn store_module_content(&mut self, id: model::module::Id, content: model::module::Content) {
        debug!("Filling frame '{}' with snapshot of module '{id}':\n{content}", self.name);
        if self.snapshots.try_insert(id, content).is_err() {
            debug!("Skipping this snapshot, as module's state was already saved.")
        }
    }

    /// Add secondary name to the frame.
    ///
    /// secondary names are used to track sub-transactions that were covered by this frame.
    pub fn add_secondary_name(&mut self, name: impl Into<String>) {
        self.secondary_names.push(name.into())
    }
}



// ==================
// === Repository ===
// ==================

/// Identifies a stack in Undo-Redo repository.
#[derive(Clone, Copy, Debug, Display, Ord, PartialOrd, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum Stack {
    Undo,
    Redo,
}

/// The inner state of the Und-Redo repository.
#[derive(Debug, Default)]
pub struct Data {
    /// Undo stack.
    pub undo:                Vec<Frame>,
    /// Redo stack.
    pub redo:                Vec<Frame>,
    /// Currently open transaction (if `Some` and alive).
    pub current_transaction: Option<Weak<Transaction>>,
}

/// `Repository` stores undo and redo stacks and provides transaction support.
///
/// This is the primary type meant to be exposed to entities that want their actions to be
/// undoable. They can group edits together by keeping a [`Transaction`] handle alive.
///
/// `Repository`, unlike [`Manager`] does not keep any modules (or other model entities) alive and
/// can be shared with no consequence on project state.
#[derive(Debug, Default)]
pub struct Repository {
    data: RefCell<Data>,
}

impl Repository {
    /// Create a new repository.
    pub fn new() -> Self {
        default()
    }

    /// Get the currently open transaction. [`None`] if there is none.
    pub fn current_transaction(&self) -> Option<Rc<Transaction>> {
        self.data.borrow().current_transaction.as_ref().and_then(Weak::upgrade)
    }

    /// Open a new transaction.
    ///
    /// If there is already an opened transaction, it will be returned as [`Err`].
    pub fn open_transaction(
        self: &Rc<Self>,
        name: impl Into<String>,
    ) -> Result<Rc<Transaction>, Rc<Transaction>> {
        if let Some(ongoing_transaction) = self.current_transaction() {
            // If there is already an ongoing transaction, we will just add a secondary name to it.
            // This is useful when we want to understand what actions were covered by given frame.
            // Apart from the debugging purposes, this deos not affect the undo-redo mechanism in
            // any way.
            let name = name.into();
            debug!(
                "Adding secondary name to transaction {}: `{name}`",
                ongoing_transaction.frame.borrow().name
            );
            ongoing_transaction.frame.borrow_mut().secondary_names.push(name);
            Err(ongoing_transaction)
        } else {
            let name = name.into();
            debug!("Creating a new transaction `{name}`");
            let new_transaction = Rc::new(Transaction::new(self, name));
            self.data.borrow_mut().current_transaction = Some(Rc::downgrade(&new_transaction));
            Ok(new_transaction)
        }
    }

    /// Open an ignored transaction.
    ///
    /// This function should be used when we want to do some changes in module which should not be
    /// tracked in undo redo (when they are not a result of user activity). If the transaction is
    /// already opened, it will **not** be ignored, and will be returned in [`Err`] variant.
    ///
    /// See also [`Repository::open_transaction`], [`Transaction::ignore`].
    pub fn open_ignored_transaction(
        self: &Rc<Self>,
        name: impl Into<String>,
    ) -> Result<Rc<Transaction>, Rc<Transaction>> {
        let transaction = self.open_transaction(name);
        if let Ok(new) = &transaction {
            new.ignore();
        }
        transaction
    }

    /// Open an ignored transaction.
    ///
    /// If there is already an ongoing transaction, it will be marked as ignored, unlike the
    /// behavior of [`Repository::open_ignored_transaction`].
    pub fn open_ignored_transaction_or_ignore_current(
        self: &Rc<Self>,
        name: impl Into<String>,
    ) -> Rc<Transaction> {
        match self.open_ignored_transaction(name) {
            Ok(transaction) => transaction,
            Err(transaction) => {
                transaction.ignore();
                transaction
            }
        }
    }

    fn new_undo_frame(&self, frame: Frame) {
        info!("Adding a new frame to the stack: {frame}. {}", backtrace());
        self.push_to(Stack::Undo, frame);
        self.clear(Stack::Redo)
    }

    /// Close the currently opened transaction.
    ///
    /// This method should not be used directly. Instead just drop the [`Transaction`] handle.
    fn close_transaction(&self, transaction: &Transaction) {
        if transaction.frame.borrow().snapshots.is_empty() {
            // If there was a transaction with no snapshots, we will just ignore it.
            // As we create transactions for every user interaction (command), there will be a lot
            // of empty transactions. We do not want to pollute the undo-redo stack with them.
            debug!("Ignoring empty transaction '{}'. It will be skipped.", transaction.name());
        } else if !transaction.ignored.get() {
            // If the transaction was not ignored, we will add it to the undo stack.
            let frame = transaction.frame.borrow().clone();
            self.new_undo_frame(frame);
        } else {
            debug!(
                "Closing the ignored transaction '{transaction}' without adding a frame to the repository.",
            )
        }
    }

    /// Get currently opened transaction. If there is none, open a new one.
    pub fn transaction(self: &Rc<Self>, name: impl Into<String>) -> Rc<Transaction> {
        match self.open_transaction(name) {
            Ok(transaction) => transaction,
            Err(transaction) => transaction,
        }
    }

    /// Borrow given stack.
    fn borrow(&self, stack: Stack) -> Ref<Vec<Frame>> {
        let data = self.data.borrow();
        match stack {
            Stack::Undo => Ref::map(data, |d| &d.undo),
            Stack::Redo => Ref::map(data, |d| &d.redo),
        }
    }

    /// Borrow given stack mutably.
    fn borrow_mut(&self, stack: Stack) -> RefMut<Vec<Frame>> {
        let data = self.data.borrow_mut();
        match stack {
            Stack::Undo => RefMut::map(data, |d| &mut d.undo),
            Stack::Redo => RefMut::map(data, |d| &mut d.redo),
        }
    }

    /// Push a new frame to the given stack.
    fn push_to(&self, stack: Stack, frame: Frame) {
        debug!(
            "Pushing to {stack} stack a new frame: {frame}. New frame count: {}",
            self.borrow(stack).len() + 1
        );
        self.borrow_mut(stack).push(frame);
    }

    /// Clear all frames from the given stack.
    fn clear(&self, stack: Stack) {
        debug!("Clearing {stack} stack, dropping {} frames.", self.borrow(stack).len(),);
        self.borrow_mut(stack).clear();
    }

    /// Clear all frames from both undo and redo stacks.
    pub fn clear_all(&self) {
        for stack in [Stack::Undo, Stack::Redo] {
            self.clear(stack)
        }
    }

    /// Get the top frame from a given stack. [`Err`] if the stack is empty.
    ///
    /// Does *not* pop.
    pub fn last(&self, stack: Stack) -> FallibleResult<Frame> {
        self.borrow(stack).last().cloned().ok_or_else(|| NoActionToUndo.into())
    }

    /// Pop the top frame from a given stack. [`Err`] if there are no frames to pop.
    fn pop(&self, stack: Stack) -> FallibleResult<Frame> {
        let frame = self.borrow_mut(stack).pop().ok_or(NoFrameToPop(stack))?;
        debug!(
            "Popping a frame from {stack}. Remaining length: {}. Frame: {frame}",
            self.len(stack)
        );
        Ok(frame)
    }

    /// Get number of frames on a given stack.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self, stack: Stack) -> usize {
        self.borrow(stack).len()
    }

    /// If there is an ongoing transaction, abort it. It will be ignored. After this call, there
    /// will be no ongoing transaction.
    ///
    /// Returns the aborted transaction, if there was one.
    pub fn abort_current_transaction(&self) -> Option<Rc<Transaction>> {
        let transaction =
            self.data.borrow_mut().current_transaction.take().and_then(|t| t.upgrade());
        if let Some(transaction) = transaction.as_ref() {
            debug!("Aborting transaction `{}`", transaction.name());
            transaction.ignore();
        }
        transaction
    }
}



// ===============
// === Manager ===
// ===============

/// Undo-Redo manager. Allows undoing or redoing recent actions.
///
/// Owns [`Repository`] and keeps track of open modules.
#[derive(Debug, Default)]
pub struct Manager {
    /// Repository with undo and redo stacks.
    pub repository: Rc<Repository>,
    /// Currently available modules.
    modules:        RefCell<BTreeMap<model::module::Id, model::Module>>,
}

impl Aware for Manager {
    fn undo_redo_repository(&self) -> Rc<Repository> {
        self.repository.clone()
    }
}

impl Manager {
    /// Create a new undo-redo manager.
    pub fn new() -> Self {
        default()
    }

    /// Register a new opened module in the manager.
    ///
    /// Only a modules registered as open can be subject of undo-redo operations.
    pub fn module_opened(&self, module: model::Module) {
        self.modules.borrow_mut().insert(module.id(), module);
    }

    /// Unregisters a previously opened module.
    pub fn module_closed(&self, module: model::Module) {
        self.modules.borrow_mut().remove(&module.id());
    }

    /// Undo last operation.
    pub fn undo(&self) -> FallibleResult {
        debug!("Undo requested, stack size is {}.", self.repository.len(Stack::Undo));
        let frame = self.repository.last(Stack::Undo)?;

        // We need to abort any ongoing transaction before applying undo. Otherwise, we might end up
        // with a situation when undoing would re-add itself onto the undo stack.
        self.repository.abort_current_transaction();

        // Before applying undo we create a special transaction. The purpose it two-fold:
        // 1) We want to prevent any undo attempt if there is already an ongoing transaction;
        // 2) We want to make sure that any of undo consequences won't create a new transaction,
        //    leading to a situation when undoing would re-add itself onto the undo stack.
        // We mark transaction as ignored right after creating, as it is never intended to create a
        // new undo frame. Instead, frame will be pushed to the redo stack manually.
        let undo_transaction = self.repository.open_transaction("Undo faux transaction").map_err(
            |ongoing_transaction| {
                let transaction_name = ongoing_transaction.name();
                CannotUndoDuringTransaction { transaction_name }
            },
        )?;
        undo_transaction.ignore();
        self.reset_to(&frame)?;
        let popped = self.repository.pop(Stack::Undo);

        // Sanity check the we popped the same frame as we have just undone. What was on top is
        // supposed to stay on top, as we maintain an open transaction while undoing.
        if !popped.contains(&frame) {
            // No reason to stop the world but should catch our eye in logs.
            error!("Undone frame mismatch!");
            debug_assert!(false, "Undone frame mismatch!");
        }

        let undo_transaction =
            Rc::try_unwrap(undo_transaction).map_err(|_| FauxTransactionLeaked)?;
        self.repository.data.borrow_mut().redo.push(undo_transaction.frame.borrow().clone());
        Ok(())
    }

    /// Redo the last undone operation.
    pub fn redo(&self) -> FallibleResult {
        let frame = self.repository.data.borrow_mut().redo.pop().ok_or(NoActionToUndo)?;
        let redo_transaction = self.get_or_open_transaction(&frame.name);
        redo_transaction.ignore();
        self.reset_to(&frame)?;
        self.repository.push_to(Stack::Undo, redo_transaction.frame.borrow().clone());
        Ok(())
    }

    /// Restore all modules affected by the [`Frame`] to their stored state.
    fn reset_to(&self, frame: &Frame) -> FallibleResult {
        info!("Resetting to initial state on frame {frame}");

        // First we must have all modules resolved. Only then we can start applying changes.
        // Otherwise, if one of the modules could not be retrieved, we'd risk ending up with
        // a partially undone operation and inconsistent state.
        //
        // In general this should never happen, as we store strong references to all opened modules
        // and don't allow getting snapshots of modules that are not opened.
        let module_and_content = with(self.modules.borrow(), |modules| {
            frame
                .snapshots
                .iter()
                .map(|(id, content)| -> FallibleResult<_> {
                    let err = || MissingModuleHandle(id.to_string());
                    let module = modules.get(id).cloned().ok_or_else(err)?;
                    Ok((module, content.clone()))
                })
                .collect::<FallibleResult<Vec<_>>>()
        })?;

        for (module, content) in module_and_content {
            info!("Undoing on module {}", module.path());
            // The below should never fail, because it can fail only if serialization to code fails.
            // And it cannot fail, as it already underwent this procedure successfully in the past
            // (we are copying an old state, so it must ba a representable state).
            module.update_whole(content.clone())?;
            // Temporary changes should not leave UR frames, but some frame could be created during
            // editing, so the temporary changes are in the snapshot. We need to remove them after
            // restoring that frame.
            module.restore_temporary_changes()?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //use utils::test::traits::*;
    use super::*;
    use crate::test::mock::Fixture;
    use crate::test::mock::Unified;
    use ast::crumbs::InfixCrumb;
    use controller::graph::Connection;
    use controller::graph::Endpoint;

    fn check_atomic_undo(fixture: Fixture, action: impl FnOnce()) {
        let Fixture { project, module, searcher, .. } = fixture;
        drop(searcher);
        let urm = project.urm();

        assert_eq!(urm.repository.len(Stack::Undo), 0);
        assert_eq!(urm.repository.len(Stack::Redo), 0);

        let before_action = module.serialized_content().unwrap();
        action();
        let after_action = module.serialized_content().unwrap();

        assert_eq!(urm.repository.len(Stack::Undo), 1);
        assert_eq!(urm.repository.len(Stack::Redo), 0);

        // After single undo everything should be as before.
        urm.undo().unwrap();
        assert_eq!(module.serialized_content().unwrap(), before_action);
        assert_eq!(urm.repository.len(Stack::Undo), 0);
        assert_eq!(urm.repository.len(Stack::Redo), 1);

        // After redo - as right after connecting.
        urm.redo().unwrap();
        assert_eq!(module.serialized_content().unwrap(), after_action);
        assert_eq!(urm.repository.len(Stack::Undo), 1);
        assert_eq!(urm.repository.len(Stack::Redo), 0);
    }

    fn check_atomic_graph_action(code: &str, action: impl FnOnce(&controller::graph::Handle)) {
        let mut data = Unified::new();
        data.set_code(code);

        let fixture = data.fixture();
        let graph = fixture.graph.clone_ref();
        check_atomic_undo(fixture, move || action(&graph));
    }

    // Collapse two middle nodes.
    #[test]
    fn collapse_nodes_atomic() {
        let code = r#"
main =
    foo = 2
    bar = foo + 6
    baz = 2 + foo + bar
    caz = baz / 2 * baz
"#;
        check_atomic_graph_action(code, |graph| {
            let nodes = graph.nodes().unwrap();
            assert_eq!(nodes.len(), 4);
            graph.collapse(vec![nodes[1].id(), nodes[2].id()], "extracted").unwrap();
        });
    }

    // A complex operation: involves introducing variable name, reordering lines and
    // replacing an argument.
    #[test]
    fn connect_nodes_atomic() {
        let code = r#"
main =
    2 + 2
    5 * 5
"#;
        check_atomic_graph_action(code, |graph| {
            let nodes = graph.nodes().unwrap();
            let sum_node = &nodes[0];
            let product_node = &nodes[1];

            assert_eq!(sum_node.expression().to_string(), "2 + 2");
            assert_eq!(product_node.expression().to_string(), "5 * 5");

            let connection = Connection {
                source: Endpoint::root(product_node.id()),
                target: Endpoint::target_at(sum_node, [InfixCrumb::LeftOperand]).unwrap(),
            };
            graph.connect(&connection, &span_tree::generate::context::Empty).unwrap();
        });
    }


    // Check that node position is properly updated.
    #[test]
    fn move_node() {
        use model::module::Position;

        let fixture = crate::test::mock::Unified::new().fixture();
        let Fixture { executed_graph, graph, project, .. } = fixture;

        let urm = project.urm();
        let nodes = executed_graph.graph().nodes().unwrap();
        let node = &nodes[0];

        debug!("{:?}", node.position());
        let pos1 = Position::new(500.0, 250.0);
        let pos2 = Position::new(300.0, 150.0);

        graph.set_node_position(node.id(), pos1).unwrap();
        graph.set_node_position(node.id(), pos2).unwrap();

        assert_eq!(graph.node(node.id()).unwrap().position(), Some(pos2));
        urm.undo().unwrap();
        assert_eq!(graph.node(node.id()).unwrap().position(), Some(pos1));
        urm.undo().unwrap();
        assert_eq!(graph.node(node.id()).unwrap().position(), None);
        urm.redo().unwrap();
        assert_eq!(graph.node(node.id()).unwrap().position(), Some(pos1));
        urm.redo().unwrap();
        assert_eq!(graph.node(node.id()).unwrap().position(), Some(pos2));
    }

    #[test]
    fn undo_redo() {
        use crate::test::mock::Fixture;
        // Setup the controller.
        let fixture = Unified::new().fixture();
        let Fixture { executed_graph, project, module, searcher, .. } = fixture;
        // Searcher makes changes in node temporary, and it affects the Undo Redo.
        drop(searcher);

        let urm = project.urm();
        let nodes = executed_graph.graph().nodes().unwrap();
        let node = &nodes[0];

        // Check initial state.
        assert_eq!(urm.repository.len(Stack::Undo), 0, "Undo stack not empty: {urm:?}");
        assert_eq!(module.ast().to_string(), "main = \n    2 + 2");

        // Perform an action.
        executed_graph.graph().set_expression(node.info.id(), "5 * 20").unwrap();

        // We can undo action.
        assert_eq!(urm.repository.len(Stack::Undo), 1);
        assert_eq!(module.ast().to_string(), "main = \n    5 * 20");
        urm.undo().unwrap();
        assert_eq!(module.ast().to_string(), "main = \n    2 + 2");

        // We cannot undo more actions than we made.
        assert_eq!(urm.repository.len(Stack::Undo), 0);
        assert!(urm.undo().is_err());
        assert_eq!(module.ast().to_string(), "main = \n    2 + 2");

        // We can redo since we undid.
        urm.redo().unwrap();
        assert_eq!(module.ast().to_string(), "main = \n    5 * 20");

        // And we can undo once more.
        urm.undo().unwrap();
        assert_eq!(module.ast().to_string(), "main = \n    2 + 2");

        //We cannot redo after edit has been made.
        executed_graph.graph().set_expression(node.info.id(), "4 * 20").unwrap();
        assert!(urm.redo().is_err());
    }
}
