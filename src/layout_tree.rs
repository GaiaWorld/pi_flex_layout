use core::marker::PhantomData;

use pi_null::Null;
use pi_slotmap::{SecondaryMap, SlotMap};
use pi_slotmap_tree::{InsertType, SlotMapTree, Storage, Tree, TreeKey};

// this declaration is necessary to "mount" the generated code where cargo can see it
// this allows us to both keep code generation scoped to a singe directory for fs events
// and to keep each test in a separate file

use crate::prelude::*;
use pi_dirty::*;

#[derive(Clone, Debug, Default)]
pub struct Style {
    pub display: Display,
    pub position_type: PositionType,
    pub direction: Direction,

    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub row_gap: f32,
    pub column_gap: f32,

    pub order: isize,
    pub flex_basis: Dimension,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub align_self: AlignSelf,

    pub overflow: Overflow,
    pub position: Rect<Dimension>,
    pub padding: Rect<Dimension>,
    pub border: Rect<Dimension>,
    pub min_size: Size<Dimension>,
    pub max_size: Size<Dimension>,
    pub aspect_ratio: Number,
    pub margin: Rect<Dimension>,
    pub size: Size<Dimension>,

	pub overflow_wrap: OverflowWrap,
	pub auto_reduce: bool,
}

impl FlexLayoutStyle for Style {
    fn width(&self) -> Dimension {
        self.size.width
    }

    fn height(&self) -> Dimension {
        self.size.height
    }

    fn margin_top(&self) -> Dimension {
        self.margin.top
    }

    fn margin_right(&self) -> Dimension {
        self.margin.right
    }

    fn margin_bottom(&self) -> Dimension {
        self.margin.bottom
    }

    fn margin_left(&self) -> Dimension {
        self.margin.left
    }

    fn padding_top(&self) -> Dimension {
        self.padding.top
    }

    fn padding_right(&self) -> Dimension {
        self.padding.right
    }

    fn padding_bottom(&self) -> Dimension {
        self.padding.bottom
    }

    fn padding_left(&self) -> Dimension {
        self.padding.left
    }

    fn position_top(&self) -> Dimension {
        self.position.top
    }

    fn position_right(&self) -> Dimension {
        self.position.right
    }

    fn position_bottom(&self) -> Dimension {
        self.position.bottom
    }

    fn position_left(&self) -> Dimension {
        self.position.left
    }

    fn border_top(&self) -> Dimension {
        self.border.top
    }

    fn border_right(&self) -> Dimension {
        self.border.right
    }

    fn border_bottom(&self) -> Dimension {
        self.border.bottom
    }

    fn border_left(&self) -> Dimension {
        self.border.left
    }

    fn display(&self) -> Display {
        self.display
    }

    fn position_type(&self) -> PositionType {
        self.position_type
    }

    fn direction(&self) -> Direction {
        self.direction
    }

    fn flex_direction(&self) -> FlexDirection {
        self.flex_direction
    }

    fn flex_wrap(&self) -> FlexWrap {
        self.flex_wrap
    }

    fn justify_content(&self) -> JustifyContent {
        self.justify_content
    }

    fn align_items(&self) -> AlignItems {
        self.align_items
    }

    fn align_content(&self) -> AlignContent {
        self.align_content
    }
    fn row_gap(&self) -> f32 {
        self.row_gap
    }
    fn column_gap(&self) -> f32{
        self.column_gap
    }

    fn order(&self) -> isize {
        self.order
    }

    fn flex_basis(&self) -> Dimension {
        self.flex_basis
    }

    fn flex_grow(&self) -> f32 {
        self.flex_grow
    }

    fn flex_shrink(&self) -> f32 {
        self.flex_shrink
    }

    fn align_self(&self) -> AlignSelf {
        self.align_self
    }

    fn overflow(&self) -> Overflow {
        self.overflow
    }

    fn min_width(&self) -> Dimension {
        self.min_size.width
    }

    fn min_height(&self) -> Dimension {
        self.min_size.height
    }

    fn max_width(&self) -> Dimension {
        self.max_size.width
    }

    fn max_height(&self) -> Dimension {
        self.max_size.height
    }

    fn aspect_ratio(&self) -> Number {
        self.aspect_ratio
    }

	fn overflow_wrap(&self) -> OverflowWrap {
		self.overflow_wrap
	}
	fn auto_reduce(&self) -> bool {
		self.auto_reduce
	}
}

#[derive(Debug, Clone, Default)]
pub struct LayoutResult {
    pub rect: Rect<f32>,
    pub border: SideGap<f32>,
    pub padding: SideGap<f32>,
    pub absolute: bool,
}


#[derive(Debug)]
pub struct LayoutResultItem<'a>(&'a mut LayoutResult);

impl<'a> LayoutR for LayoutResultItem<'a> {
    fn rect(&self) -> &Rect<f32> {
        &self.0.rect
    }

    fn border(&self) -> &SideGap<f32> {
        &self.0.border
    }

    fn padding(&self) -> &SideGap<f32> {
        &self.0.padding
    }
    fn absolute(&self) -> bool {
        self.0.absolute
    }

    fn set_rect(&mut self, v: Rect<f32>) {
        self.0.rect = v;
    }

    fn set_border(&mut self, v: SideGap<f32>) {
        self.0.border = v;
    }

    fn set_padding(&mut self, v: SideGap<f32>) {
        self.0.padding = v;
    }

    fn set_absolute(&mut self, b: bool) {
        self.0.absolute = b;
    }

    fn set_finish(&mut self) {}
}

pub struct LayoutTree {
    pub style_map: Styles,
    pub layout_map: SecondaryMap<TreeKey, LayoutResult>,
    pub nodes: SlotMap<TreeKey, ()>,
    pub tree: SlotTreeStorage,
    pub dirty: LayerDirty<TreeKey>,

    pub node_states: SecondaryMap<TreeKey, INode>,
}

pub struct Styles(pub SecondaryMap<TreeKey, Style>);

impl Get<TreeKey> for Styles {
    type Target = Style;

    fn get(&self, k: TreeKey) -> Self::Target {
        self.0.get(k).unwrap().clone()
    }
}

impl GetMut<TreeKey> for Styles {
    type Target = Style;

    fn get_mut(&mut self, k: TreeKey) -> Self::Target {
        self.0.get(k).unwrap().clone()
    }
}

pub struct LayoutResults<'a>(pub &'a mut SecondaryMap<TreeKey, LayoutResult>);

impl<'a> Get<TreeKey> for LayoutResults<'a> {
    type Target = LayoutResultItem<'a>;

    fn get(&self, k: TreeKey) -> Self::Target {
        if let None = self.0.get(k) {
            let s = unsafe { &mut *(self as *const Self as usize as *mut Self) };
            s.0.insert(k, LayoutResult::default());
        }
        LayoutResultItem(unsafe {
            &mut *(self.0.get(k).unwrap() as *const LayoutResult as usize as *mut LayoutResult)
        })
    }
}

impl<'a> GetMut<TreeKey> for LayoutResults<'a> {
    type Target = LayoutResultItem<'a>;

    fn get_mut(&mut self, k: TreeKey) -> Self::Target {
        if let None = self.0.get(k) {
            self.0.insert(k, LayoutResult::default());
        }
        LayoutResultItem(unsafe {
            &mut *(self.0.get(k).unwrap() as *const LayoutResult as usize as *mut LayoutResult)
        })
    }
}

pub struct SlotTreeStorage(pub Tree<TreeKey, SlotMapTree>);
impl TreeStorage<TreeKey> for SlotTreeStorage {
    fn get_up(&self, id: TreeKey) -> Option<pi_slotmap_tree::Up<TreeKey>> {
        match self.0.get_up(id) {
            Some(r) => Some(r.clone()),
            None => None,
        }
    }

    fn get_down(&self, id: TreeKey) -> Option<pi_slotmap_tree::Down<TreeKey>> {
        match self.0.get_down(id) {
            Some(r) => Some(r.clone()),
            None => None,
        }
    }

    fn up(&self, id: TreeKey) -> pi_slotmap_tree::Up<TreeKey> {
        self.0.up(id).clone()
    }

    fn down(&self, id: TreeKey) -> pi_slotmap_tree::Down<TreeKey> {
        self.0.down(id).clone()
    }

    fn get_layer(&self, id: TreeKey) -> Option<usize> {
        match self.0.get_layer(id) {
            Some(r) => Some(r.layer()),
            None => None,
        }
    }

    fn layer(&self, id: TreeKey) -> usize {
        self.0.layer(id).layer()
    }
}

impl Default for LayoutTree {
    fn default() -> Self {
        Self {
            tree: SlotTreeStorage(Tree::new(SlotMapTree::default())),
            style_map: Styles(SecondaryMap::default()),
            layout_map: SecondaryMap::default(),
            nodes: SlotMap::default(),
            dirty: LayerDirty::default(),
            node_states: SecondaryMap::default(),
        }
    }
}

impl LayoutTree {
    pub fn create_node(&mut self) -> TreeKey {
        self.nodes.insert(())
    }
    pub fn get_layout(&self, key: TreeKey) -> Option<&LayoutResult> {
        self.layout_map.get(key)
    }
    pub fn insert(
        &mut self,
        id: TreeKey,
        parent: TreeKey,
        brother: TreeKey,
        insert: InsertType,
        s: Style,
    ) {
        if !brother.is_null() {
            self.tree.0.insert_brother(id, brother, insert);
        } else {
            self.tree.0.insert_child(id, parent, std::usize::MAX);
        }
        self.node_states.insert(id, INode::default());
        self.set_style(id, s);
        self.layout_map.insert(id, LayoutResult::default());
    }

    pub fn set_style(&mut self, id: TreeKey, s: Style) {
        if s.display == Display::None {
            self.style_map.0.insert(id, s.clone());
            return;
        }
        self.style_map.0.insert(id, s.clone());
        let c = LayoutContext {
            mark: PhantomData,
            i_nodes: &mut self.node_states,
            layout_map: &mut LayoutResults(&mut self.layout_map),
            notify_arg: &mut (),
            notify: notify,
            tree: &mut self.tree,
            style: &mut self.style_map,
        };
        let mut layout = Layout(c);
        layout.set_rect(&mut self.dirty, id, true, true,  &s);
    }
}

pub fn notify(_arg: &mut (), _k: TreeKey, _r: &LayoutResultItem) {}
