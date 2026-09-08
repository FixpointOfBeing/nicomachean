use crate::liveness_rv_var::{RvVarBasicBlockLiveness, RvVarInstrLiveness, liveness_analysis};
use crate::riscv::{FReg, XReg};
use crate::riscv_var::RvVarBasicBlock;
use crate::riscv_var::RvVarInstr;
use crate::riscv_var::RvVarLocation;
use crate::riscv_var::RvVarProgram;
use crate::riscv_var::location::x;
use petgraph::{graph::NodeIndex, graph::UnGraph};
use priority_queue::PriorityQueue;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::usize;

struct RvVarLocationGraph {
    ungraph: UnGraph<RvVarLocation, RvVarLocation>,
    location_nodes: HashMap<RvVarLocation, NodeIndex>,
}

impl RvVarLocationGraph {
    fn new() -> Self {
        let ungraph = UnGraph::<RvVarLocation, RvVarLocation>::new_undirected();
        let location_nodes = HashMap::new();
        RvVarLocationGraph { ungraph, location_nodes }
    }

    fn add_location(&mut self, location: &RvVarLocation) -> NodeIndex {
        let idx = self.ungraph.add_node(location.clone());
        self.location_nodes.insert(location.clone(), idx);
        idx
    }

    fn interfere(&mut self, location1: &RvVarLocation, location2: &RvVarLocation) {
        if location1 == location2 {
            return;
        }

        if is_float_location(location1) && is_x_location(location2) {
            return;
        }

        if is_float_location(location2) && is_x_location(location1) {
            return;
        }

        if location1 == &RvVarLocation::XReg(XReg::ZERO) || location2 == &RvVarLocation::XReg(XReg::ZERO) {
            return;
        }

        let node1 = match self.location_nodes.get(location1) {
            Some(idx) => idx.clone(),
            None => self.add_location(location1),
        };

        let node2 = match self.location_nodes.get(location2) {
            Some(idx) => idx.clone(),
            None => self.add_location(location2),
        };
        let edge_location_name = format!("{}<->{}", location1, location2);

        if !self.ungraph.contains_edge(node1, node2) {
            self.ungraph
                .add_edge(node1, node2, RvVarLocation::Dummy(edge_location_name));
        }
    }

    fn neighbors(&self, location: &RvVarLocation) -> Vec<RvVarLocation> {
        if let Some(idx) = self.location_nodes.get(location) {
            let mut neighbors = Vec::new();
            for adj_idx in self.ungraph.neighbors(*idx) {
                neighbors.push(self.ungraph[adj_idx].clone());
            }

            neighbors
        } else {
            Vec::new()
        }
    }

    fn all_locations(&self) -> HashSet<RvVarLocation> {
        self.location_nodes.keys().cloned().collect()
    }
}

// todo:
/*
*死定义（dead def）会产生多余干涉边
 一个定义后从不再被用的变量，仍会与它的 live-in 操作数连边。这不会导致错误代码，但会过度约束 → 可能多 spill，降低着色质量。可先做死代码消除，或对「不在任何后续 live 集合里的 def」跳过
*/

pub fn build_infer_graph(block: &RvVarBasicBlockLiveness) -> (RvVarLocationGraph, RvVarLocationGraph) {
    let mut x_graph = RvVarLocationGraph::new();
    let mut f_graph = RvVarLocationGraph::new();
    for instr in &block.instrs {
        add_write_live_edge(&mut x_graph, &mut f_graph, instr);
    }
    (x_graph, f_graph)
}

fn is_float_location(location: &RvVarLocation) -> bool {
    matches!(location, RvVarLocation::FVar(_) | RvVarLocation::FReg(_))
}

fn is_x_location(location: &RvVarLocation) -> bool {
    matches!(location, RvVarLocation::XVar(_) | RvVarLocation::XReg(_))
}

fn add_write_live_edge(x_graph: &mut RvVarLocationGraph, f_graph: &mut RvVarLocationGraph, instr: &RvVarInstrLiveness) {
    if let Some(write) = instr.instr.dest_location() {
        if write == RvVarLocation::XReg(XReg::ZERO) {
            return;
        }
        if is_x_location(&write) {
            for live in &instr.live {
                if is_x_location(live) {
                    x_graph.interfere(&write, live);
                }
            }
        } else if is_float_location(&write) {
            for live in &instr.live {
                if is_float_location(live) {
                    f_graph.interfere(&write, live);
                }
            }
        }
    }
}

const ALLOCATABLE_XREGS_SIZE: u8 = 26;

const ALLOCATABLE_XREG_NUMS: [u8; ALLOCATABLE_XREGS_SIZE as usize] = [
    5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
];

//  ZERO(x0): 恒为 0。 RA(x1): 返回地址。SP(x2): 栈指针。GP(x3): 全局指针。TP(x4):线程指针。FP(x8): 帧指针。
const NON_ALLOCATABLE_XREGS_SIZE: u8 = 6;

const NON_ALLOCATABLE_XREG_NUMS: [u8; NON_ALLOCATABLE_XREGS_SIZE as usize] = [0, 1, 2, 3, 4, 8];

const ALLOCATABLE_FREGS_SIZE: u8 = 32;

const ALLOCATABLE_FREG_NUMS: [u8; ALLOCATABLE_FREGS_SIZE as usize] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
    31,
];

fn color_to_xreg(color: u8) -> XReg {
    assert!((1..=ALLOCATABLE_XREGS_SIZE).contains(&color));
    XReg::from_u8(ALLOCATABLE_XREG_NUMS[(color - 1) as usize])
}

fn color_to_freg(color: u8) -> FReg {
    assert!((1..=ALLOCATABLE_FREGS_SIZE).contains(&color));
    FReg::from_u8(ALLOCATABLE_FREG_NUMS[(color - 1) as usize])
}

fn freg_to_color(reg: &FReg) -> i8 {
    (reg.num() + 1) as i8
}

fn xreg_to_color(reg: &XReg) -> i8 {
    match reg.num() {
        0 => -1,
        1 => -2,
        2 => -3,
        3 => -4,
        4 => -5,
        8 => -9,

        5 => 1,
        6 => 2,
        7 => 3,
        9 => 4,
        10 => 5,
        11 => 6,
        12 => 7,
        13 => 8,
        14 => 9,
        15 => 10,
        16 => 11,
        17 => 12,
        18 => 13,
        19 => 14,
        20 => 15,
        21 => 16,
        22 => 17,
        23 => 18,
        24 => 19,
        25 => 20,
        26 => 21,
        27 => 22,
        28 => 23,
        29 => 24,
        30 => 25,
        31 => 26,

        _ => unreachable!(),
    }
}

fn allocatable_xregs() -> HashSet<RvVarLocation> {
    let regs = vec![
        x(4),
        x(5),
        x(6),
        x(7),
        x(9),
        x(10),
        x(11),
        x(12),
        x(13),
        x(14),
        x(15),
        x(16),
        x(17),
        x(18),
        x(19),
        x(20),
        x(21),
        x(22),
        x(23),
        x(24),
        x(25),
        x(26),
        x(27),
        x(28),
        x(29),
        x(30),
        x(31),
    ];
    regs.into_iter().collect()
}

type Color = i8;
const UNCOLORED: i8 = 0;

fn color_graph(
    x_graph: &RvVarLocationGraph,
    f_graph: &RvVarLocationGraph,
) -> (HashMap<RvVarLocation, Color>, HashMap<RvVarLocation, Color>) {
    let x = color_one_class(x_graph, |loc| match loc {
        RvVarLocation::XReg(r) => Some(xreg_to_color(r)),
        _ => None,
    });
    let f = color_one_class(f_graph, |loc| match loc {
        RvVarLocation::FReg(r) => Some(freg_to_color(r)),
        _ => None,
    });
    (x, f)
}

fn color_one_class<F>(graph: &RvVarLocationGraph, reg_to_color: F) -> HashMap<RvVarLocation, Color>
where
    F: Fn(&RvVarLocation) -> Option<Color>,
{
    // 分类：物理寄存器 → Some(色)；变量 → None
    let var_locations: HashSet<RvVarLocation> = graph
        .all_locations()
        .into_iter()
        .filter(|loc| reg_to_color(loc).is_none())
        .collect();
    let mut var_color = HashMap::new();
    let mut var_saturation = HashMap::new();

    // 初始化：变量 UNCOLORED，saturation 吸收邻居（预着色寄存器 / 其他变量）的颜色
    for location in &var_locations {
        var_color.insert(location.clone(), UNCOLORED);
        let mut sat = BTreeSet::new();
        for neighbor in graph.neighbors(location) {
            let c = match reg_to_color(&neighbor) {
                Some(c) => c,
                None => *var_color.get(&neighbor).unwrap_or(&UNCOLORED),
            };
            if c != UNCOLORED {
                sat.insert(c);
            }
        }
        var_saturation.insert(location.clone(), sat);
    }

    // DSATUR 着色
    let mut queue = PriorityQueue::new();
    for location in &var_locations {
        queue.push(location.clone(), var_saturation[location].len());
    }
    while let Some((location, _)) = queue.pop() {
        assert!(var_color.get(&location) == Some(&UNCOLORED));
        let mut color = UNCOLORED + 1;
        while var_saturation[&location].contains(&color) {
            color += 1;
        }

        // 更新color
        var_color.insert(location.clone(), color);

        // 更新neighbor的staturation
        for neighbor in graph.neighbors(&location) {
            if let Some(sat) = var_saturation.get_mut(&neighbor) {
                sat.insert(color);
                queue.change_priority(&neighbor, sat.len());
            }
        }
    }
    var_color
}

// fn color_graph(
//     x_graph: &RvVarLocationGraph,
//     f_graph: &RvVarLocationGraph,
// ) -> (HashMap<RvVarLocation, Color>, HashMap<RvVarLocation, Color>) {
//     let mut x_var_color = HashMap::<RvVarLocation, Color>::new();
//     let mut f_var_color = HashMap::<RvVarLocation, Color>::new();
//     let mut x_var_staturation = HashMap::<RvVarLocation, BTreeSet<Color>>::new();
//     let mut f_var_staturation = HashMap::<RvVarLocation, BTreeSet<Color>>::new();
//
//     let all_x_locations = x_graph.all_locations();
//     let all_f_locations = f_graph.all_locations();
//
//     let mut all_x_var_locations = HashSet::<RvVarLocation>::new();
//     let mut all_f_var_locations = HashSet::<RvVarLocation>::new();
//     let mut all_x_reg_locations = HashSet::<RvVarLocation>::new();
//     let mut all_f_reg_locations = HashSet::<RvVarLocation>::new();
//
//     for location in &all_x_locations {
//         match location {
//             RvVarLocation::Dummy(_) => {},
//             RvVarLocation::StackSlot { .. } => {},
//
//             RvVarLocation::XVar(_) => {
//                 all_x_var_locations.insert(location.clone());
//             },
//             RvVarLocation::FVar(_) => {},
//             RvVarLocation::XReg(_) => {
//                 all_x_reg_locations.insert(location.clone());
//             },
//             RvVarLocation::FReg(_) => {},
//         }
//     }
//
//     for location in &all_f_locations {
//         match location {
//             RvVarLocation::Dummy(_) => {},
//             RvVarLocation::StackSlot { .. } => {},
//
//             RvVarLocation::XVar(_) => {},
//             RvVarLocation::FVar(_) => {
//                 all_f_var_locations.insert(location.clone());
//             },
//             RvVarLocation::XReg(_) => {},
//             RvVarLocation::FReg(_) => {
//                 all_f_reg_locations.insert(location.clone());
//             },
//         }
//     }
//
//     // 初始化XVar的color和saturation
//     for location in &all_x_var_locations {
//         x_var_color.insert(location.clone(), UNCOLORED);
//
//         let mut staturation = BTreeSet::<Color>::new();
//         for neighbor in x_graph.neighbors(location) {
//             let neighbor_color = if all_x_reg_locations.contains(&neighbor) {
//                 if let RvVarLocation::XReg(xreg) = &neighbor { xreg_to_color(xreg) } else { unreachable!() }
//             } else {
//                 let neighbor_color = x_var_color.get(&neighbor).unwrap_or(&UNCOLORED);
//                 *neighbor_color
//             };
//             if neighbor_color != UNCOLORED {
//                 staturation.insert(neighbor_color);
//             }
//         }
//         x_var_staturation.insert(location.clone(), staturation);
//     }
//
//     // 初始化FVar的color和saturation
//     for location in &all_f_var_locations {
//         f_var_color.insert(location.clone(), UNCOLORED);
//         let mut staturation = BTreeSet::<Color>::new();
//         for neighbor in f_graph.neighbors(location) {
//             let neighbor_color = if all_f_reg_locations.contains(&neighbor) {
//                 if let RvVarLocation::FReg(reg) = &neighbor { freg_to_color(reg) } else { unreachable!() }
//             } else {
//                 let neighbor_color = f_var_color.get(&neighbor).unwrap_or(&UNCOLORED);
//                 *neighbor_color
//             };
//             if neighbor_color != UNCOLORED {
//                 staturation.insert(neighbor_color);
//             }
//         }
//         f_var_staturation.insert(location.clone(), staturation);
//     }
//
//     // 给XVar着色
//     let mut x_variable_queue = PriorityQueue::<RvVarLocation, usize>::new();
//     for location in &all_x_var_locations {
//         if let Some(saturation) = x_var_staturation.get(location) {
//             x_variable_queue.push(location.clone(), saturation.len());
//         }
//     }
//
//     while !x_variable_queue.is_empty() {
//         if let Some((location, _)) = x_variable_queue.pop() {
//             assert!(x_var_color.get(&location) == Some(&UNCOLORED));
//             if let Some(staturation) = x_var_staturation.get(&location) {
//                 let mut color = UNCOLORED + 1;
//                 while staturation.contains(&color) {
//                     color += 1;
//                 }
//
//                 // 更新neighbor的staturation
//                 for neighbor in x_graph.neighbors(&location) {
//                     if let Some(neighbor_staturation) = x_var_staturation.get_mut(&neighbor) {
//                         neighbor_staturation.insert(color);
//                         x_variable_queue.change_priority(&neighbor, neighbor_staturation.len());
//                     }
//                 }
//                 // 更新color
//                 x_var_color.insert(location, color);
//             }
//         }
//     }
//
//     // 给FVar着色
//     let mut f_variable_queue = PriorityQueue::<RvVarLocation, usize>::new();
//     for location in &all_f_var_locations {
//         if let Some(saturation) = f_var_staturation.get(location) {
//             f_variable_queue.push(location.clone(), saturation.len());
//         }
//     }
//
//     while !f_variable_queue.is_empty() {
//         if let Some((location, _)) = f_variable_queue.pop() {
//             assert!(f_var_color.get(&location) == Some(&UNCOLORED));
//             if let Some(staturation) = f_var_staturation.get(&location) {
//                 let mut color = UNCOLORED + 1;
//                 while staturation.contains(&color) {
//                     color += 1;
//                 }
//
//                 // 更新neighbor的staturation
//                 for neighbor in f_graph.neighbors(&location) {
//                     if let Some(neighbor_staturation) = f_var_staturation.get_mut(&neighbor) {
//                         neighbor_staturation.insert(color);
//                         f_variable_queue.change_priority(&neighbor, neighbor_staturation.len());
//                     }
//                 }
//                 // 更新color
//                 f_var_color.insert(location, color);
//             }
//         }
//     }
//
//     (x_var_color, f_var_color)
// }

const LOCATION_SIZE: u8 = 8;
fn location_to_reg(
    location: RvVarLocation,
    x_var_color: &HashMap<RvVarLocation, Color>,
    f_var_color: &HashMap<RvVarLocation, Color>,
) -> RvVarLocation {
    match location {
        RvVarLocation::XVar(_) => {
            let color = x_var_color[&location];
            assert!(color > 0);
            let color = color as u8;
            if color < ALLOCATABLE_XREGS_SIZE + 1 {
                // color 从1开始
                let xreg = color_to_xreg(color);
                RvVarLocation::XReg(xreg)
            } else {
                let offset = (LOCATION_SIZE as i32) * (color - ALLOCATABLE_XREGS_SIZE) as i32;
                RvVarLocation::StackSlot { offset, size: LOCATION_SIZE as u32 }
            }
        },
        RvVarLocation::FVar(_) => {
            let color = f_var_color[&location];
            assert!(color > 0);
            let color = color as u8;
            if color < ALLOCATABLE_FREGS_SIZE + 1 {
                // color 从1开始
                let freg = color_to_freg(color);
                RvVarLocation::FReg(freg)
            } else {
                let offset = (LOCATION_SIZE as i32) * (color - ALLOCATABLE_FREGS_SIZE) as i32;
                RvVarLocation::StackSlot { offset, size: LOCATION_SIZE as u32 }
            }
        },
        RvVarLocation::Dummy(_) => location,
        RvVarLocation::XReg(_) => location,
        RvVarLocation::FReg(_) => location,
        RvVarLocation::StackSlot { .. } => location,
    }
}

fn allocate_block(block: RvVarBasicBlock) -> RvVarBasicBlock {
    let block_liveness = liveness_analysis(&block);
    let (x_graph, f_graph) = build_infer_graph(&block_liveness);
    let (x_var_color, f_var_color) = color_graph(&x_graph, &f_graph);

    let mut instrs = Vec::<RvVarInstr>::new();
    for instr in block.instrs {
        let mut map_dest = |location: RvVarLocation| location_to_reg(location, &x_var_color, &f_var_color);
        let mut map_src = |location: RvVarLocation| location_to_reg(location, &x_var_color, &f_var_color);
        instrs.push(instr.map_operands(&mut map_dest, &mut map_src));
    }
    RvVarBasicBlock { name: block.name.clone(), instrs }
}

pub fn allocate_registers(var_prog: RvVarProgram) -> RvVarProgram {
    let mut prog = RvVarProgram::new();
    for basic_block in var_prog.blocks {
        prog.append_basic_block(allocate_block(basic_block));
    }
    prog
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::riscv::rv64imfd_instr::Rm;
    use crate::riscv::rv64imfd_reg::XReg;
    use crate::riscv_var::instruction::RvVarInstr;
    use crate::riscv_var::location::{fvar, var, x0};
    use std::collections::HashSet;

    type Graph = UnGraph<RvVarLocation, RvVarLocation>;

    fn ivar(name: &str) -> RvVarLocation {
        var(name.to_string())
    }

    fn fv(name: &str) -> RvVarLocation {
        fvar(name.to_string())
    }

    fn locs(locations: &[RvVarLocation]) -> HashSet<RvVarLocation> {
        locations.iter().cloned().collect()
    }

    fn il(instr: RvVarInstr, live: &[RvVarLocation]) -> RvVarInstrLiveness {
        RvVarInstrLiveness { instr, live: locs(live) }
    }

    fn graph_of(instrs: Vec<RvVarInstrLiveness>) -> (RvVarLocationGraph, RvVarLocationGraph) {
        let block = RvVarBasicBlockLiveness {
            name: crate::riscv::Label::new("bb".to_string()),
            instrs,
            live_out: HashSet::new(),
        };
        build_infer_graph(&block)
    }

    fn f_graph_of(instrs: Vec<RvVarInstrLiveness>) -> Graph {
        let (_, f_graph) = graph_of(instrs);
        f_graph.ungraph
    }

    fn x_graph_of(instrs: Vec<RvVarInstrLiveness>) -> Graph {
        let (x_graph, _) = graph_of(instrs);
        x_graph.ungraph
    }

    fn node(g: &Graph, location: &RvVarLocation) -> Option<NodeIndex> {
        g.node_indices().find(|&i| g.node_weight(i) == Some(location))
    }

    fn has_edge(g: &Graph, a: &RvVarLocation, b: &RvVarLocation) -> bool {
        match (node(g, a), node(g, b)) {
            (Some(x), Some(y)) => g.contains_edge(x, y),
            _ => false,
        }
    }

    #[test]
    fn int_def_links_live_int_sources() {
        let g = x_graph_of(vec![il(
            RvVarInstr::Add { rd: ivar("a"), rs1: ivar("b"), rs2: ivar("c") },
            &[ivar("b"), ivar("c")],
        )]);
        assert!(has_edge(&g, &ivar("a"), &ivar("b")));
        assert!(has_edge(&g, &ivar("a"), &ivar("c")));
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn float_def_links_live_float_sources() {
        let g = f_graph_of(vec![il(
            RvVarInstr::FaddS { rd: fv("d"), rs1: fv("x"), rs2: fv("y"), rm: Rm::Rne },
            &[fv("x"), fv("y")],
        )]);
        assert!(has_edge(&g, &fv("d"), &fv("x")));
        assert!(has_edge(&g, &fv("d"), &fv("y")));
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn no_cross_class_edge() {
        // int def reading float source (fmv.x.w)
        let g = x_graph_of(vec![il(RvVarInstr::FmvXW { rd: ivar("d"), rs1: fv("f") }, &[fv("f")])]);
        assert!(!has_edge(&g, &ivar("d"), &fv("f")));
        assert_eq!(g.edge_count(), 0);

        // float def reading int source (fmv.w.x)
        let g = x_graph_of(vec![il(RvVarInstr::FmvWX { rd: fv("d"), rs1: ivar("i") }, &[ivar("i")])]);
        assert!(!has_edge(&g, &fv("d"), &ivar("i")));
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn zero_def_produces_nothing() {
        let g = x_graph_of(vec![il(
            RvVarInstr::Add { rd: x0(), rs1: ivar("a"), rs2: ivar("b") },
            &[ivar("a"), ivar("b")],
        )]);
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn zero_source_is_ignored() {
        let g = x_graph_of(vec![il(
            RvVarInstr::Add { rd: ivar("d"), rs1: x0(), rs2: ivar("b") },
            &[x0(), ivar("b")],
        )]);
        assert!(has_edge(&g, &ivar("d"), &ivar("b")));
        assert!(!has_edge(&g, &ivar("d"), &x0()));
        assert!(node(&g, &x0()).is_none());
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn self_edge_is_skipped() {
        let g = x_graph_of(vec![il(
            RvVarInstr::Add { rd: ivar("a"), rs1: ivar("a"), rs2: ivar("b") },
            &[ivar("a"), ivar("b")],
        )]);
        assert!(has_edge(&g, &ivar("a"), &ivar("b")));
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn int_var_interferes_with_physical_int_reg() {
        let g = x_graph_of(vec![il(
            RvVarInstr::Add { rd: ivar("a"), rs1: RvVarLocation::XReg(XReg::A0), rs2: ivar("c") },
            &[RvVarLocation::XReg(XReg::A0), ivar("c")],
        )]);
        assert!(has_edge(&g, &ivar("a"), &RvVarLocation::XReg(XReg::A0)));
        assert!(has_edge(&g, &ivar("a"), &ivar("c")));
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn mixed_int_and_float_stay_separate() {
        let (x_g, f_g) = graph_of(vec![
            il(RvVarInstr::Add { rd: ivar("a"), rs1: ivar("b"), rs2: ivar("c") }, &[ivar("b"), ivar("c")]),
            il(RvVarInstr::FaddS { rd: fv("d"), rs1: fv("x"), rs2: fv("y"), rm: Rm::Rne }, &[fv("x"), fv("y")]),
        ]);
        let x_g = x_g.ungraph;
        let f_g = f_g.ungraph;
        assert!(has_edge(&x_g, &ivar("a"), &ivar("b")));
        assert!(has_edge(&f_g, &fv("d"), &fv("x")));
        assert!(!has_edge(&x_g, &ivar("a"), &fv("d")));
        assert!(!has_edge(&f_g, &ivar("b"), &fv("x")));
        assert_eq!((x_g.edge_count() + f_g.edge_count()), 4);
    }

    #[test]
    fn spills_to_stack_when_x_registers_exhausted() {
        use crate::riscv::rv64imfd_imm::Imm12;

        // 27 个同时活跃的整数变量构成 27 团，超过 26 个可分配 X 寄存器，必须溢出。
        let mut instrs = Vec::new();
        for i in 0..27 {
            instrs.push(RvVarInstr::Addi { rd: ivar(&format!("v{i}")), rs1: x0(), imm: Imm12::from_i16(1) });
        }
        let acc = ivar("acc");
        instrs.push(RvVarInstr::Add { rd: acc.clone(), rs1: ivar("v0"), rs2: ivar("v1") });
        for i in 2..27 {
            instrs.push(RvVarInstr::Add { rd: acc.clone(), rs1: acc.clone(), rs2: ivar(&format!("v{i}")) });
        }

        let mut prog = RvVarProgram::new();
        prog.append_basic_block(RvVarBasicBlock { name: crate::riscv::Label::new("bb".to_string()), instrs });

        // println!("{}", prog);
        let allocated = allocate_registers(prog);
        // println!("{}", allocated);
    }
}
