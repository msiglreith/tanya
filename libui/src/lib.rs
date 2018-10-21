#[macro_use]
extern crate yoga;
extern crate indextree;
extern crate random_color;

use indextree::Arena;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;
use yoga::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Unique(usize),
    Local { parent: usize, child: usize },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceId(usize);

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

pub struct Instance {
    widget: Box<Widget>,
    pub layout: LayoutNode,
    pub geometry: Rect,
}

pub type LayoutNode = Rc<RefCell<yoga::Node>>;

type InstanceTree = Arena<WidgetId>;
pub type NodeId = indextree::NodeId;

pub struct IdGenerator {
    next_id: usize,
}

impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator { next_id: 1 }
    }

    pub fn gen_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

pub struct Ui {
    states: HashMap<Key, Box<Any>>,
    pub tree: InstanceTree,
    pub instances: HashMap<WidgetId, Instance>,
    pub instance_lut: HashMap<WidgetId, NodeId>,
    id_gen: IdGenerator,
    key_gen: IdGenerator,
    pub root: WidgetId,
}

impl Ui {
    pub fn new() -> Self {
        let layout = unsafe { yoga::Node::new() };
        Ui {
            states: HashMap::new(),
            instances: HashMap::new(),
            instance_lut: HashMap::new(),
            tree: InstanceTree::new(),
            id_gen: IdGenerator::new(),
            key_gen: IdGenerator::new(),
            root: WidgetId(!0),
        }
    }

    pub fn build<W: Widget>(&mut self, widget: W) {
        let key = self.gen_key();
        let id = {
            let context = BuilderContext {
                instances: &mut self.instances,
                instance_lut: &mut self.instance_lut,
                tree: &mut self.tree,
                states: &mut self.states,
                id_gen: &mut self.id_gen,
            };

            widget.build(context, Some(key))
        };

        let instance = Instance {
            widget: Box::new(widget),
            layout: Rc::new(RefCell::new(yoga::Node::new())),
            geometry: Rect {
                left: 0.0,
                right: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
        };
        self.instances.insert(id, instance);
        self.root = id;
    }

    pub fn layout(&mut self, width: f32, height: f32) {
        {
            let root = &self.instances[&self.root];
            root.widget.layout(LayoutContext {
                id: self.instance_lut[&self.root],
                tree: &self.tree,
                instances: &self.instances,
            });

            root.layout
                .borrow_mut()
                .calculate_layout(width, height, yoga::Direction::LTR);
        }

        let base_geometry = Rect {
            left: 0.0,
            right: width,
            top: 0.0,
            bottom: height,
        };

        let root_idx = self.instance_lut[&self.root];
        let mut geometry_context = GeometryContext {
            tree: &self.tree,
            instances: &mut self.instances,
        };
        geometry_context.calculate_geometry(root_idx, base_geometry);
    }

    pub fn gen_key(&mut self) -> Key {
        Key::Unique(self.key_gen.gen_id())
    }
}

struct GeometryContext<'a> {
    tree: &'a InstanceTree,
    instances: &'a mut HashMap<WidgetId, Instance>,
}

impl<'a> GeometryContext<'a> {
    fn calculate_geometry(mut self, idx: NodeId, parent: Rect) {
        let geometry = {
            let id = self.tree[idx].data;
            let mut instance = self.instances.get_mut(&id).unwrap();
            let layout = instance.layout.borrow_mut().get_layout();
            let left = parent.left + layout.left();
            let top = parent.top + layout.top();
            let geometry = Rect {
                left,
                right: left + layout.width(),
                top,
                bottom: top + layout.height(),
            };

            instance.geometry = geometry;
            geometry
        };
        for child in idx.children(&self.tree) {
            let ctxt = GeometryContext {
                tree: &self.tree,
                instances: &mut self.instances,
            };
            ctxt.calculate_geometry(child, geometry);
        }
    }
}

pub struct BuilderContext<'a> {
    instances: &'a mut HashMap<WidgetId, Instance>,
    instance_lut: &'a mut HashMap<WidgetId, NodeId>,
    tree: &'a mut InstanceTree,
    states: &'a mut HashMap<Key, Box<Any>>,
    id_gen: &'a mut IdGenerator,
}

impl<'a> BuilderContext<'a> {
    pub fn gen_id(&mut self) -> WidgetId {
        WidgetId(self.id_gen.gen_id())
    }

    pub fn add<W: Widget>(&mut self, widget: W, key: Option<Key>) -> WidgetId {
        let id = {
            let context = BuilderContext {
                instances: &mut self.instances,
                instance_lut: &mut self.instance_lut,
                tree: &mut self.tree,
                states: &mut self.states,
                id_gen: &mut self.id_gen,
            };

            widget.build(context, key)
        };
        let instance = Instance {
            widget: Box::new(widget),
            layout: Rc::new(RefCell::new(yoga::Node::new())),
            geometry: Rect {
                left: 0.0,
                right: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
        };
        self.instances.insert(id, instance);
        println!("add {:?}", id);
        id
    }
}

pub struct LayoutContext<'a> {
    id: NodeId,
    tree: &'a InstanceTree,
    instances: &'a HashMap<WidgetId, Instance>,
}

pub struct RenderContext {}

pub trait Widget: 'static {
    fn build(&self, ctxt: BuilderContext, key: Option<Key>) -> WidgetId;
    fn layout(&self, ctxt: LayoutContext);
    fn render(&self, ctxt: RenderContext);
}

pub trait StatefulWidget {
    type State;

    fn create_state(&self) -> Self::State;
}

pub trait State: Sized {
    type Widget: StatefulWidget<State = Self>;
    type Repr: Widget;

    fn build(&self, ctxt: BuilderContext, widget: &Self::Widget) -> Self::Repr;
}

pub trait StatelessWidget {
    type Repr: Widget;
    fn build(&self) -> Self::Repr;
}

pub struct Button {}

impl Widget for Button {
    fn build(&self, mut ctxt: BuilderContext, key: Option<Key>) -> WidgetId {
        let id = ctxt.gen_id();
        let key = key.expect("Stateful widgets need an key");
        let (key, key_id) = if let Key::Unique(key_id) = key {
            (key, key_id)
        } else {
            panic!("Unique key required") // TODO: handle local
        };

        let state = if !ctxt.states.contains_key(&key) {
            let state = self.create_state();
            Box::new(state)
        } else {
            ctxt.states
                .remove(&key)
                .unwrap()
                .downcast::<<Self as StatefulWidget>::State>()
                .unwrap()
        };

        let repr = {
            let context = BuilderContext {
                instances: &mut ctxt.instances,
                instance_lut: &mut ctxt.instance_lut,
                tree: &mut ctxt.tree,
                states: &mut ctxt.states,
                id_gen: &mut ctxt.id_gen,
            };
            state.build(context, self)
        };

        ctxt.states.insert(key, state);

        let child_id = ctxt.add(
            repr,
            Some(Key::Local {
                parent: key_id,
                child: 0,
            }),
        );

        let tree_parent = ctxt.tree.new_node(id);
        ctxt.instance_lut.insert(id, tree_parent);
        let tree_child = ctxt.instance_lut[&child_id];
        let _ = tree_parent.append(tree_child, &mut ctxt.tree);

        id
    }

    fn layout(&self, ctxt: LayoutContext) {
        println!("layout Button {:?}", ctxt.id);
        let id = ctxt.tree[ctxt.id].data;
        let instance = &ctxt.instances[&id];
        let mut num_child = 0;

        let child_styles = make_styles!(FlexGrow(1.0), Margin(10 pt));
        instance.layout.borrow_mut().apply_styles(&child_styles);

        for child in ctxt.id.children(&ctxt.tree) {
            let child_id = ctxt.tree[child].data;
            let child_instance = &ctxt.instances[&child_id];
            instance
                .layout
                .borrow_mut()
                .insert_child(&mut *child_instance.layout.borrow_mut(), num_child);
            num_child += 1;
            child_instance.widget.layout(LayoutContext {
                id: child,
                tree: &ctxt.tree,
                instances: &ctxt.instances,
            });
        }
    }

    fn render(&self, ctxt: RenderContext) {}
}

impl StatefulWidget for Button {
    type State = ButtonState;

    fn create_state(&self) -> Self::State {
        ButtonState {}
    }
}

pub struct ButtonState {}

impl State for ButtonState {
    type Widget = Button;
    type Repr = Label;

    fn build(&self, ctxt: BuilderContext, widget: &Self::Widget) -> Self::Repr {
        Label {}
    }
}

pub struct Label {}

impl Widget for Label {
    fn build(&self, mut ctxt: BuilderContext, id: Option<Key>) -> WidgetId {
        let id = ctxt.gen_id();
        let tree = ctxt.tree.new_node(id);
        ctxt.instance_lut.insert(id, tree);
        println!("build label {:?}", (id, tree));
        id
    }

    fn layout(&self, ctxt: LayoutContext) {
        println!("layout Label");

        let id = ctxt.tree[ctxt.id].data;
        let instance = &ctxt.instances[&id];
        let child_styles = make_styles!(FlexGrow(1.0), Margin(10 pt));
        instance.layout.borrow_mut().apply_styles(&child_styles);
    }

    fn render(&self, ctxt: RenderContext) {}
}

pub struct Row {
    widgets: Vec<WidgetId>,
}

impl Widget for Row {
    fn build(&self, mut ctxt: BuilderContext, id: Option<Key>) -> WidgetId {
        let id = ctxt.gen_id();
        let tree_parent = ctxt.tree.new_node(id);
        ctxt.instance_lut.insert(id, tree_parent);

        for widget in &self.widgets {
            let tree_child = ctxt.instance_lut[&widget];
            let _ = tree_parent.append(tree_child, &mut ctxt.tree);
        }

        id
    }

    fn layout(&self, ctxt: LayoutContext) {
        println!("layout Row");
        let id = ctxt.tree[ctxt.id].data;
        let instance = &ctxt.instances[&id];
        let mut num_child = 0;

        let child_styles = make_styles!(
            FlexGrow(1.0),
            Margin(5 pt),
            FlexDirection(yoga::FlexDirection::Row)
        );
        instance.layout.borrow_mut().apply_styles(&child_styles);

        for child in ctxt.id.children(&ctxt.tree) {
            println!("row {:?}", (id, child));
            let child_id = ctxt.tree[child].data;
            let child_instance = &ctxt.instances[&child_id];
            instance
                .layout
                .borrow_mut()
                .insert_child(&mut *child_instance.layout.borrow_mut(), num_child);

            num_child += 1;
            child_instance.widget.layout(LayoutContext {
                id: child,
                tree: &ctxt.tree,
                instances: &ctxt.instances,
            });
        }
    }

    fn render(&self, ctxt: RenderContext) {}
}

// TODO: move into example
pub struct App {
    pub button0: Key,
    pub button1: Key,
}

impl Widget for App {
    fn build(&self, mut ctxt: BuilderContext, key: Option<Key>) -> WidgetId {
        let id = ctxt.gen_id();
        let key = key.expect("Stateful widgets need an key");
        let (key, key_id) = if let Key::Unique(key_id) = key {
            (key, key_id)
        } else {
            panic!("Unique key required")
        };

        let state = if !ctxt.states.contains_key(&key) {
            let state = self.create_state();
            Box::new(state)
        } else {
            ctxt.states
                .remove(&key)
                .unwrap()
                .downcast::<<Self as StatefulWidget>::State>()
                .unwrap()
        };

        let repr = {
            let context = BuilderContext {
                instances: &mut ctxt.instances,
                instance_lut: &mut ctxt.instance_lut,
                tree: &mut ctxt.tree,
                states: &mut ctxt.states,
                id_gen: &mut ctxt.id_gen,
            };
            state.build(context, self)
        };

        ctxt.states.insert(key, state);

        let child_id = ctxt.add(
            repr,
            Some(Key::Local {
                parent: key_id,
                child: 0,
            }),
        );

        let tree_parent = ctxt.tree.new_node(id);
        ctxt.instance_lut.insert(id, tree_parent);
        let tree_child = ctxt.instance_lut[&child_id];
        let _ = tree_parent.append(tree_child, &mut ctxt.tree);

        id
    }

    fn layout(&self, mut ctxt: LayoutContext) {
        println!("layout app");
        let id = ctxt.tree[ctxt.id].data;
        let instance = &ctxt.instances[&id];
        let mut num_child = 0;
        for child in ctxt.id.children(&ctxt.tree) {
            let child_id = ctxt.tree[child].data;
            let child_instance = &ctxt.instances[&child_id];
            println!("app {:?}", (id, child_id));
            instance
                .layout
                .borrow_mut()
                .insert_child(&mut *child_instance.layout.borrow_mut(), num_child);
            num_child += 1;
            child_instance.widget.layout(LayoutContext {
                id: child,
                tree: &ctxt.tree,
                instances: &ctxt.instances,
            });
        }
    }

    fn render(&self, ctxt: RenderContext) {}
}

impl StatefulWidget for App {
    type State = AppState;

    fn create_state(&self) -> Self::State {
        AppState {}
    }
}

pub struct AppState {}

impl State for AppState {
    type Widget = App;
    type Repr = Row;

    fn build(&self, mut ctxt: BuilderContext, widget: &Self::Widget) -> Self::Repr {
        let button0 = ctxt.add(Button {}, Some(widget.button0));
        let button1 = ctxt.add(Label {}, Some(widget.button1));
        Row {
            widgets: vec![button0, button1],
        }
    }
}
