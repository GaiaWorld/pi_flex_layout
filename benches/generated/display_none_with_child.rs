fn print < T : pi_flex_layout :: prelude :: LayoutR + std :: fmt :: Debug > (_arg : & mut () , id : pi_slotmap_tree :: TreeKey , layout : & T) { println ! ("result: {:?} {:?}" , id , layout) ; } pub fn compute () { let mut layout_tree = pi_flex_layout :: prelude :: LayoutTree :: default () ; let node_1 = layout_tree . create_node () ; layout_tree . insert (node_1 , < pi_slotmap_tree :: TreeKey as pi_null :: Null > :: null () , < pi_slotmap_tree :: TreeKey as pi_null :: Null > :: null () , pi_slotmap_tree :: InsertType :: Back , pi_flex_layout :: prelude :: Style { position_type : pi_flex_layout :: prelude :: PositionType :: Absolute , size : pi_flex_layout :: prelude :: Size { width : pi_flex_layout :: prelude :: Dimension :: Points (1920.0) , height : pi_flex_layout :: prelude :: Dimension :: Points (1024.0) , } , position : pi_flex_layout :: prelude :: Rect { left : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , right : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , top : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , bottom : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , } , margin : pi_flex_layout :: prelude :: Rect { left : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , right : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , top : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , bottom : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , } , .. Default :: default () }) ; let node_2 = layout_tree . create_node () ; layout_tree . insert (node_2 , node_1 , < pi_slotmap_tree :: TreeKey as pi_null :: Null > :: null () , pi_slotmap_tree :: InsertType :: Back , pi_flex_layout :: prelude :: Style { size : pi_flex_layout :: prelude :: Size { width : pi_flex_layout :: prelude :: Dimension :: Points (100f32) , height : pi_flex_layout :: prelude :: Dimension :: Points (100f32) , .. Default :: default () } , .. Default :: default () } ,) ; let node_3 = layout_tree . create_node () ; layout_tree . insert (node_3 , node_2 , < pi_slotmap_tree :: TreeKey as pi_null :: Null > :: null () , pi_slotmap_tree :: InsertType :: Back , pi_flex_layout :: prelude :: Style { flex_grow : 1f32 , flex_shrink : 1f32 , flex_basis : pi_flex_layout :: prelude :: Dimension :: Percent (0f32) , .. Default :: default () } ,) ; let node_4 = layout_tree . create_node () ; layout_tree . insert (node_4 , node_2 , < pi_slotmap_tree :: TreeKey as pi_null :: Null > :: null () , pi_slotmap_tree :: InsertType :: Back , pi_flex_layout :: prelude :: Style { display : pi_flex_layout :: prelude :: Display :: None , flex_direction : pi_flex_layout :: prelude :: FlexDirection :: Column , flex_grow : 1f32 , flex_shrink : 1f32 , flex_basis : pi_flex_layout :: prelude :: Dimension :: Percent (0f32) , .. Default :: default () } ,) ; let node_5 = layout_tree . create_node () ; layout_tree . insert (node_5 , node_4 , < pi_slotmap_tree :: TreeKey as pi_null :: Null > :: null () , pi_slotmap_tree :: InsertType :: Back , pi_flex_layout :: prelude :: Style { flex_grow : 1f32 , flex_shrink : 1f32 , flex_basis : pi_flex_layout :: prelude :: Dimension :: Percent (0f32) , size : pi_flex_layout :: prelude :: Size { width : pi_flex_layout :: prelude :: Dimension :: Points (20f32) , .. Default :: default () } , .. Default :: default () } ,) ; let node_6 = layout_tree . create_node () ; layout_tree . insert (node_6 , node_2 , < pi_slotmap_tree :: TreeKey as pi_null :: Null > :: null () , pi_slotmap_tree :: InsertType :: Back , pi_flex_layout :: prelude :: Style { flex_grow : 1f32 , flex_shrink : 1f32 , flex_basis : pi_flex_layout :: prelude :: Dimension :: Percent (0f32) , .. Default :: default () } ,) ; layout_tree . compute (print , & mut 0) ; }