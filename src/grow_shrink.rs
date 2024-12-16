
//! 根据flex布局， 子节点如果有min或max约束了grow和shrink，则按有min或max约束的子节点的grow-shrink来先计算一趟缩放。然后再根据余出来的空间，再进行多轮迭代，继续将没有达到约束上限的节点计算缩放。直到所有约束都完毕，然后再在无约束的子节点的grow-shrink来计算一次缩放。
//! 注意， 收缩和扩展不同，根据css规范组， shrink的权重是shrink * basis，可能是css规范组希望等比收缩，这样不会出现收缩成负值
//! grow的值如果小于0，并且总的grow值也小于1，则表示每个grow仅扩展指定的百分比，最后会有剩余空间
//! shrink的情况和grow类似，shrink的值如果小于0，并且总的shrink值也小于1，则表示每个shrink仅收缩shrink * basis权重对应的百分比，最后会有溢出空间


use crate::calc::RelNodeInfo;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Data {
    grow: f32,
    shrink: f32,
    min: Option<f32>,
    max: Option<f32>,
    basis: Option<f32>,
    length: f32,
    result: f32,
    result_maybe_ok: bool,
}
impl Data {
    pub fn get_real_basis(&self) -> f32 {
        let b = self.basis.unwrap_or(self.length);
        self.get_max_basis(self.get_min_basis(b))
    }

    pub fn get_min_basis(&self, basis: f32) -> f32 {
        match self.min {
            Some(min) if basis < min => min,
            _ => basis,
        }
    }
    pub fn get_max_basis(&self, basis: f32) -> f32 {
        match self.max {
            Some(max) if basis > max => max,
            _ => basis,
        }
    }
    // 统计
    pub fn statistics(&self, context: &mut GrowShrinkContext) {
        let basis = self.get_real_basis();
        context.basis += basis;
        if self.grow > 0.0 {
            context.grow_weight += self.grow;
            context.grow_basis += basis;
            match self.max {
                Some(max) => {
                    context.max_grow_amount += max;
                }
                None => {
                    context.only_grow_count += 1;
                }
            }
        }else{
            // 如果没有设置grow，则basis统计到context.no_grow_basis上
            context.no_grow_basis += basis;
        }
        if self.shrink > 0.0 {
            // 注意， 收缩和扩展不同，根据css规范组， shrink的权重是shrink * basis，可能是css规范组希望等比收缩，这样不会出现收缩成负值
            context.shrink_weight += self.shrink * basis;
            context.shrink_basis += basis;
            match self.min {
                Some(min) => {
                    context.min_shrink_amount += min;
                }
                None => (),
            }
        } else {
            // 如果没有设置shrink，则basis统计到context.no_shrink_basis上
            context.no_shrink_basis += basis;
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
pub struct GrowShrinkContext {
    /// 统计的总grow权重值
    pub(crate) grow_weight: f32,
    /// 统计的有grow的总basis值
    pub(crate) grow_basis: f32,
    /// 统计的无max有grow的子节点数量
    pub(crate) only_grow_count: u32,
    // 统计的有max有grow的总max值
    pub(crate) max_grow_amount: f32,
    /// 统计的无grow的总值
    pub(crate) no_grow_basis: f32,

    /// 统计的总shrink权重值
    pub(crate) shrink_weight: f32,
    /// 统计的有shrink的总basis值
    pub(crate) shrink_basis: f32,
    /// 统计的有min有shrink的总min值
    pub(crate) min_shrink_amount: f32,
    /// 统计的无shrink的总值
    pub(crate) no_shrink_basis: f32,
    /// 统计的总值
    pub(crate) basis: f32,
    /// 当前容器的值
    length: f32,
    /// 当前容器的值
    /// main: f32,
    /// 子节点的总值
    pub(crate) amount: f32,
    /// 计算时的总权重值
    pub(crate) weight: f32,
    /// 计算时的总权重值对应的参数值
    pub(crate) weight_basis: f32,
    // /// 统计时， basis值累计超过容器值的子节点位置索引， // 用于计算时，判断在那折行
    // basis_exceed_index: usize,
}
impl GrowShrinkContext {
    // 根据grow、shrink和min、max、basis，进行统计
    pub fn statistics_array(&mut self, array: &[Data]) {
        // 第一趟扫描，统计
        for (i, data) in array.iter().enumerate() {
            data.statistics(self);
            // if self.basis < self.length {
            //     self.basis_exceed_index = i;
            // }
        }
    }
    pub fn statistics<K>(&mut self, el: RelNodeInfo<K>) {
        let basis = self.get_real_basis();
        self.basis += basis;
        if el.grow > 0.0 {
            self.grow_weight += el.grow;
            self.grow_basis += basis;
            match el.max_main {
                Defined(max) => {
                    self.max_grow_amount += max;
                }
                _ => {
                    self.only_grow_count += 1;
                }
            }
        }else{
            // 如果没有设置grow，则basis统计到self.no_grow_basis上
            self.no_grow_basis += basis;
        }
        if el.shrink > 0.0 {
            // 注意， 收缩和扩展不同，根据css规范组， shrink的权重是shrink * basis，可能是css规范组希望等比收缩
            self.shrink_weight += el.shrink * basis;
            self.shrink_basis += basis;
            match el.min_main {
                Defined(min) => {
                    self.min_shrink_amount += min;
                }
                _ => (),
            }
        } else {
            // 如果没有设置shrink，则basis统计到self.no_shrink_basis上
            self.no_shrink_basis += basis;
        }
    }

    // 根据统计值，及每子节点的grow、shrink和min、max、basis，计算当前值
    pub fn calculate(&mut self, array: &mut [Data], main: f32) {
        if self.basis == main {
            self.amount = self.basis;
            return self.set_basis(array);
        }
        // 表示basis都超过了容器值, 需要收缩
        if self.basis > main {
            // 表示没有需要收缩的子节点，直接设置basis值
            if self.shrink_weight == 0.0 {
                self.amount = self.basis;
                return self.set_basis(array);
            }
            // 表示最小值都超过了容器值, 则每个可收缩节点都取最小值
            if self.no_shrink_basis + self.min_shrink_amount >= main {
                self.amount = self.no_shrink_basis + self.min_shrink_amount;
                return self.set_min(array);
            }
            // 表示没有 有min的节点，那就只有可收缩并且无min节点，按比例计算收缩
            if self.min_shrink_amount == 0.0 {
                self.weight = self.shrink_weight;
                self.weight_basis = main - self.no_shrink_basis - self.shrink_basis;
                self.amount = main;
                return self.shrink(array);
            }
            // 表示有可收缩的空间, 则每个可收缩节点按约束及比例多轮计算收缩
            return self.shrink_min(array, main);
        }
        // 表示basis小于容器值, 可扩展
        // 子节点没有设置扩展，仅设置basis
        if self.grow_weight == 0.0 {
            self.amount = self.basis;
            return self.set_basis(array);
        }
        // 所有可扩展的节点都有最大值，并且可扩展的节点的最大值小于容器值
        if self.only_grow_count == 0 && self.max_grow_amount + self.no_grow_basis <= main {
            self.amount = self.no_grow_basis + self.max_grow_amount;
            return self.set_max(array);
        }
        // 表示没有有max的节点，那就只有可扩展并且无max节点，按比例计算扩展
        if self.max_grow_amount == 0.0 {
            self.weight = self.grow_weight;
            self.weight_basis = main - self.no_grow_basis - self.grow_basis;
            self.amount = main;
            return self.grow(array);
        }
        // 表示有可扩展的空间, 则每个可扩展节点按约束及比例多轮计算扩展
        self.grow_max(array, main)
    }
    // 表示最小值都超过了容器值, 则每个可收缩的节点都取最小值
    pub fn set_min(&mut self, array: &mut [Data]) {
        for el in array.iter_mut() {
            let r = if el.shrink == 0.0 {
                // 该节点不需要收缩，则取basis值
                el.get_real_basis()
            } else {
                // 该节点需要收缩，则取min值
                el.min.unwrap_or(0.0)
            };
            el.result = r;
        }
    }
    // 表示最大值都超过了容器值, 则每个可扩展的节点都取最大值
    pub fn set_max(&mut self, array: &mut [Data]) {
        for el in array.iter_mut() {
            let r = if el.grow == 0.0 {
                // 该节点不需要扩展，则取basis值
                el.get_real_basis()
            } else {
                // 该节点需要扩展，则取max值
                el.max.unwrap()
            };
            el.result = r;
        }
    }

    // 表示子节点没有设置收缩和扩展, 则每个节点都取basis值
    pub fn set_basis(&mut self, array: &mut [Data]) {
        for el in array.iter_mut() {
            el.result = el.get_real_basis();
        }
    }
    // 表示有子节点需要收缩
    pub fn shrink(&mut self, array: &mut [Data]) {
        for el in array.iter_mut() {
            let r = if el.shrink > 0.0 {
                // 该节点需要收缩， 计算 shrink的值
                let b = el.get_real_basis();
                b + el.shrink * b * self.weight_basis / self.weight
            } else {
                el.get_real_basis()
            };
            el.result = r;
        }
    }
    // 表示有子节点需要扩展
    pub fn grow(&mut self, array: &mut [Data]) {
        for el in array.iter_mut() {
            let r = if el.grow > 0.0 {
                // 该节点需要扩展， 计算 grow的值
                let b = el.get_real_basis();
                b - el.grow * self.weight_basis / self.weight
            } else {
                // 该节点不需要扩展，则取basis值
                el.get_real_basis()
            };
            el.result = r;
        }
    }
    // 表示有子节点需要在min约束下收缩
    pub fn shrink_min(&mut self, array: &mut [Data], main: f32) {
        let mut fix_weight = 0.0;
        let mut fix_basis = self.no_shrink_basis;
        let mut re_basis = 0.0;
        let mut re_calc = false;
        let weight = self.shrink_weight;
        let weight_basis = main - fix_basis - self.shrink_basis;
        // 第一轮，将所有不收缩的节点计算结果，并计算收缩节点
        for el in array.iter_mut() {
            if el.shrink == 0.0 {
                // 该节点不需要收缩，则取basis值
                el.result = el.get_real_basis();
                self.amount += el.result;
                continue;
            }
            // 该节点需要收缩， 计算 shrink的值
            let b = el.get_real_basis();
            let r = b + el.shrink * b * weight_basis / weight;
            let r = match el.min {
                Some(min) if r <= min => {
                    re_calc = true;
                    fix_weight += el.shrink * b;
                    fix_basis += min;
                    min
                }
                _ => {
                    el.result_maybe_ok = true;
                    re_basis += b;
                    r
                }
            };
            el.result = r;
            self.amount += el.result;
        }
        // println!("self.shrink:{} fix_weight:{}  fix_basis:{} re_basis:{} re_calc:{}", self.shrink, fix_weight, fix_basis, re_basis, re_calc);
        while re_calc {
            let weight = self.shrink_weight - fix_weight;
            let weight_basis = self.length - fix_basis - re_basis;
            re_basis = 0.0;
            re_calc = false;
            // 多轮计算有min的收缩节点
            for el in array.iter_mut() {
                if !el.result_maybe_ok {
                    // 该节点已经计算过，跳过
                    continue;
                }
                self.amount -= el.result;
                // 该节点需要收缩， 计算 shrink后的值
                let b = el.get_real_basis();
                let r = b + el.shrink * b * weight_basis / weight;
                let r = match el.min {
                    Some(min) if r < min => {
                        el.result_maybe_ok = false;
                        re_calc = true;
                        fix_weight += el.shrink * b;
                        fix_basis += min;
                        min
                    }
                    _ => {
                        re_basis += b;
                        r
                    }
                };
                el.result = r;
                self.amount += el.result;
            }
        }
    }
    // 表示有子节点需要扩展，并且有子节点有最大值，需要按顺序重新计算权重
    pub fn grow_max(&mut self, array: &mut [Data], main: f32) {
        let mut fix_weight = 0.0;
        let mut fix_basis = self.no_grow_basis;
        let mut re_basis = 0.0;
        let mut re_calc = false;
        let weight = self.grow_weight;
        let weight_basis = main - fix_basis - self.grow_basis;
        // 第一轮，将所有不扩展的节点计算结果，并计算扩展节点
        for el in array.iter_mut() {
            if el.grow == 0.0 {
                // 该节点不需要扩展，则取basis值
                el.result = el.get_real_basis();
                self.amount += el.result;
                continue;
            }
            // 该节点需要扩展， 计算 grow后的值
            let b = el.get_real_basis();
            let r = b + el.grow * weight_basis / weight;
            let r = match el.max {
                Some(max) if r >= max => {
                    re_calc = true;
                    fix_weight += el.grow;
                    fix_basis += max;
                    max
                }
                _ => {
                    el.result_maybe_ok = true;
                    re_basis += b;
                    r
                }
            };
            el.result = r;
            self.amount += el.result;
        }
        // println!("fix_weight:{} re_basis:{} re_calc:{}", fix_weight, re_basis, re_calc);
        while re_calc {
            let weight = self.grow_weight - fix_weight;
            let weight_basis = main - fix_basis - re_basis;
            re_basis = 0.0;
            re_calc = false;
            // 多轮计算有max的扩展节点
            for el in array.iter_mut() {
                if !el.result_maybe_ok {
                    // 该节点已经计算过，跳过
                    continue;
                }
                self.amount -= el.result;
                // 该节点需要扩展， 计算 grow后的值
                let b = el.get_real_basis();
                let r = b + el.grow * weight_basis / weight;
                let r = match el.max {
                    Some(max) if r > max => {
                        el.result_maybe_ok = false;
                        re_calc = true;
                        fix_weight += el.grow;
                        fix_basis += max;
                        max
                    }
                    _ => {
                        re_basis += b;
                        r
                    }
                };
                el.result = r;
                self.amount += el.result;
            }
        }
    }
}

#[cfg(test)]
mod test_mod {
    use crate::grow_shrink::*;
    use rand::{Rng, SeedableRng};
    use pcg_rand::*;

    #[test]
    fn test_grow() {
        for i in 1..2000 {
            
            let mut rng = Pcg32::seed_from_u64(i);
            let mut array = vec![ Data::default(); 3];
            let arr = [0.0, 1.0, 2.0];
            for el in array.iter_mut() {
                el.length = rng.gen_range(0..50) as f32;
                el.min = Some(rng.gen_range(0..30) as f32);
                el.max = Some(rng.gen_range(el.min.unwrap() as usize..100) as f32);
                el.grow = arr[rng.gen_range(0..3)];
            }
            let mut con = GrowShrinkContext::default();
            con.statistics_array(&array.as_slice());
            con.calculate(&mut array.as_mut_slice(), 100.0);
            dbg!(&array);
            let mut amount = 0.0;
            for el in array.iter() {
                assert!(el.result <= el.max.unwrap());
                assert!(el.result >= el.min.unwrap());
                amount += el.result;
            }
            dbg!(i, &con, amount);
            let r = (amount - con.amount).abs();
            assert!(r <= 0.0001);
            if con.grow_weight == 0.0 {
                assert_eq!(amount, con.no_grow_basis);
            }else if con.no_grow_basis + con.grow_basis >= con.length {// 收缩
                assert_eq!(amount, con.no_grow_basis + con.grow_basis);
            }else if con.no_grow_basis + con.max_grow_amount >= con.length {
                let r = (amount - con.length).abs();
                    assert!(r <= 0.0001);
            }else{// 扩展到最大值
                assert_eq!(amount, con.no_grow_basis + con.max_grow_amount);
            }
        }

    }
    #[test]
    fn test_shrink() {
        for i in 1..3000 {
            
            let mut rng = Pcg32::seed_from_u64(i);
            let mut array = vec![ Data::default(); 3];
            let arr = [0.0, 1.0, 2.0];
            for el in array.iter_mut() {
                el.length = rng.gen_range(20..80) as f32;
                el.min = Some(rng.gen_range(20..50) as f32);
                el.max = Some(rng.gen_range(el.min.unwrap() as usize..100) as f32);
                el.shrink = arr[rng.gen_range(0..3)];
            }
            let mut con = GrowShrinkContext::default();
            con.statistics_array(&array.as_slice());
            con.calculate(&mut array.as_mut_slice(), 100.0);
            // dbg!(&array);
            let mut amount = 0.0;
            for el in array.iter() {
                assert!(el.result <= el.max.unwrap());
                assert!(el.result >= el.min.unwrap());
                amount += el.result;
            }
            dbg!(i, &con, amount);
            let r = (amount - con.amount).abs();
            assert!(r <= 0.0001);
            if con.shrink_weight == 0.0 {
                assert_eq!(amount, con.no_shrink_basis);
            }else if con.no_shrink_basis + con.shrink_basis <= con.length {// 扩展
                assert_eq!(amount, con.no_shrink_basis + con.shrink_basis);
            }else if con.no_shrink_basis + con.min_shrink_amount <= con.length {
                let r = (amount - con.length).abs();
                    assert!(r <= 0.0001);
            }else{// 收缩到最小值
                assert_eq!(amount, con.no_shrink_basis + con.min_shrink_amount);
            }
        }

    }
    #[test]
    fn test() {
        for i in 1..2000 {
            
            let mut rng = Pcg32::seed_from_u64(i);
            let mut array = vec![ Data::default(); 3];
            let arr = [0.0, 1.0, 2.0];
            for el in array.iter_mut() {
                el.length = rng.gen_range(0..100) as f32;
                el.min = Some(rng.gen_range(0..50) as f32);
                el.max = Some(rng.gen_range(el.min.unwrap() as usize..110) as f32);
                el.shrink = arr[rng.gen_range(0..3)];
                el.grow = arr[rng.gen_range(0..3)];
            }
            let mut con = GrowShrinkContext::default();
            con.statistics_array(&array.as_slice());
            con.calculate(&mut array.as_mut_slice(), 100.0);
            // dbg!(&array);
            dbg!(i, &con);
            let mut amount = 0.0;
            for el in array.iter() {
                assert!(el.result <= el.max.unwrap());
                assert!(el.result >= el.min.unwrap());
                amount += el.result;
            }
            dbg!(amount);
            let r = (amount - con.amount).abs();
            assert!(r <= 0.0001);
            if con.basis < con.length { // 扩展
                assert!(amount <= con.length + 0.01);
            }else if con.basis > con.length { // 收缩
                assert!(amount >=con.length - 0.01);
            }
        }
    }
}