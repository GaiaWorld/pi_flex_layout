fn print < T : pi_flex_layout :: prelude :: LayoutR + std :: fmt :: Debug > (_arg : & mut () , id : pi_slotmap_tree :: TreeKey , layout : & T) { unsafe { debugit :: debugit ! ("result: {:?} {:?}" , id , layout) } ; } pub fn compute () { let mut layout_tree = pi_flex_layout :: prelude :: LayoutTree :: default () ; let node_1 = layout_tree . create_node () ; layout_tree . insert (node_1 , < pi_slotmap_tree :: TreeKey as pi_null :: Null > :: null () , < pi_slotmap_tree :: TreeKey as pi_null :: Null > :: null () , pi_slotmap_tree :: InsertType :: Back , pi_flex_layout :: prelude :: Style { position_type : pi_flex_layout :: prelude :: PositionType :: Absolute , size : pi_flex_layout :: prelude :: Size { width : pi_flex_layout :: prelude :: Dimension :: Points (1920.0) , height : pi_flex_layout :: prelude :: Dimension :: Points (1024.0) , } , position : pi_flex_layout :: prelude :: Rect { left : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , right : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , top : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , bottom : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , } , margin : pi_flex_layout :: prelude :: Rect { left : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , right : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , top : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , bottom : pi_flex_layout :: prelude :: Dimension :: Points (0.0) , } , .. Default :: default () }) ; let node_2 = layout_tree . create_node () ; layout_tree . insert (node_2 , node_1 , < pi_slotmap_tree :: TreeKey as pi_null :: Null > :: null () , pi_slotmap_tree :: InsertType :: Back , pi_flex_layout :: prelude :: Style { size : pi_flex_layout :: prelude :: Size { width : pi_flex_layout :: prelude :: Dimension :: Points (100f32) , height : pi_flex_layout :: prelude :: Dimension :: Points (100f32) , .. Default :: default () } , .. Default :: default () } ,) ; let node_3 = layout_tree . create_node () ; layout_tree . insert (node_3 , node_2 , < pi_slotmap_tree :: TreeKey as pi_null :: Null > :: null () , pi_slotmap_tree :: InsertType :: Back , pi_flex_layout :: prelude :: Style { position_type : pi_flex_layout :: prelude :: PositionType :: Absolute , position : pi_flex_layout :: prelude :: Rect { left : pi_flex_layout :: prelude :: Dimension :: Points (10f32) , right : pi_flex_layout :: prelude :: Dimension :: Points (10f32) , top : pi_flex_layout :: prelude :: Dimension :: Points (10f32) , bottom : pi_flex_layout :: prelude :: Dimension :: Points (10f32) , .. Default :: default () } , .. Default :: default () } ,) ; layout_tree . compute (print , & mut 0) ; }