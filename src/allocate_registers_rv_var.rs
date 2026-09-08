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

const ALLOCATABLE_FREGS_SIZE: u8 = 32;

const ALLOCATABLE_FREG_NUMS: [u8; ALLOCATABLE_FREGS_SIZE as usize] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
    31,
];

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
    let mut x_var_color = HashMap::<RvVarLocation, Color>::new();
    let mut f_var_color = HashMap::<RvVarLocation, Color>::new();
    let mut x_var_staturation = HashMap::<RvVarLocation, BTreeSet<Color>>::new();
    let mut f_var_staturation = HashMap::<RvVarLocation, BTreeSet<Color>>::new();

    let all_x_locations = x_graph.all_locations();
    let all_f_locations = f_graph.all_locations();

    let mut x_var_locations = HashSet::<RvVarLocation>::new();
    let mut f_var_locations = HashSet::<RvVarLocation>::new();
    let mut x_reg_locations = HashSet::<RvVarLocation>::new();
    let mut f_reg_locations = HashSet::<RvVarLocation>::new();

    for location in &all_x_locations {
        match location {
            RvVarLocation::Dummy(_) => {},
            RvVarLocation::StackSlot { .. } => {},

            RvVarLocation::XVar(_) => {
                x_var_locations.insert(location.clone());
            },
            RvVarLocation::FVar(_) => {},
            RvVarLocation::XReg(_) => {
                x_reg_locations.insert(location.clone());
            },
            RvVarLocation::FReg(_) => {},
        }
    }

    for location in &all_f_locations {
        match location {
            RvVarLocation::Dummy(_) => {},
            RvVarLocation::StackSlot { .. } => {},

            RvVarLocation::XVar(_) => {},
            RvVarLocation::FVar(_) => {
                f_var_locations.insert(location.clone());
            },
            RvVarLocation::XReg(_) => {},
            RvVarLocation::FReg(_) => {
                f_reg_locations.insert(location.clone());
            },
        }
    }

    // 初始化XVar的color和saturation
    for location in &x_var_locations {
        x_var_color.insert(location.clone(), UNCOLORED);

        let mut staturation = BTreeSet::<Color>::new();
        for neighbor in x_graph.neighbors(location) {
            if x_reg_locations.contains(&neighbor) {
                let neighbor_color = x_var_color[&neighbor];
                if neighbor_color != UNCOLORED {
                    staturation.insert(neighbor_color);
                }
            }
        }
        x_var_staturation.insert(location.clone(), staturation);
    }

    // 初始化FVar的color和saturation
    for location in &f_var_locations {
        f_var_color.insert(location.clone(), UNCOLORED);
        let mut staturation = BTreeSet::<Color>::new();
        for neighbor in x_graph.neighbors(location) {
            if f_reg_locations.contains(&neighbor) {
                let neighbor_color = f_var_color[&neighbor];
                if neighbor_color != UNCOLORED {
                    staturation.insert(neighbor_color);
                }
            }
        }
        f_var_staturation.insert(location.clone(), staturation);
    }

    // 给XVar着色
    let mut x_variable_queue = PriorityQueue::<RvVarLocation, usize>::new();
    for location in &x_var_locations {
        if let Some(saturation) = x_var_staturation.get(location) {
            x_variable_queue.push(location.clone(), saturation.len());
        }
    }

    while !x_variable_queue.is_empty() {
        if let Some((location, _)) = x_variable_queue.pop() {
            assert!(x_var_color.get(&location) == Some(&UNCOLORED));
            if let Some(staturation) = x_var_staturation.get(&location) {
                let mut color = UNCOLORED + 1;
                while staturation.contains(&color) {
                    color += 1;
                }

                // 更新neighbor的staturation
                for neighbor in x_graph.neighbors(&location) {
                    if let Some(neighbor_staturation) = x_var_staturation.get_mut(&neighbor) {
                        neighbor_staturation.insert(color);
                    }
                }
                // 更新color
                x_var_color.insert(location, color);
            }
        }
    }

    // 给FVar着色
    let mut f_variable_queue = PriorityQueue::<RvVarLocation, usize>::new();
    for location in &f_var_locations {
        if let Some(saturation) = f_var_staturation.get(location) {
            f_variable_queue.push(location.clone(), saturation.len());
        }
    }

    while !f_variable_queue.is_empty() {
        if let Some((location, _)) = f_variable_queue.pop() {
            assert!(f_var_color.get(&location) == Some(&UNCOLORED));
            if let Some(staturation) = f_var_staturation.get(&location) {
                let mut color = UNCOLORED + 1;
                while staturation.contains(&color) {
                    color += 1;
                }

                // 更新neighbor的staturation
                for neighbor in x_graph.neighbors(&location) {
                    if let Some(neighbor_staturation) = f_var_staturation.get_mut(&neighbor) {
                        neighbor_staturation.insert(color);
                    }
                }
                // 更新color
                f_var_color.insert(location, color);
            }
        }
    }

    (x_var_color, f_var_color)
}

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
            if color < ALLOCATABLE_XREGS_SIZE {
                let xreg = XReg::from_u8(ALLOCATABLE_XREG_NUMS[color as usize]);
                RvVarLocation::XReg(xreg)
            } else {
                let offset = (LOCATION_SIZE as i32) * (color - ALLOCATABLE_XREGS_SIZE + 1) as i32;
                RvVarLocation::StackSlot { offset, size: LOCATION_SIZE as u32 }
            }
        },
        RvVarLocation::FVar(_) => {
            let color = f_var_color[&location];
            assert!(color > 0);
            let color = color as u8;
            if color < ALLOCATABLE_FREGS_SIZE {
                let freg = FReg::from_u8(ALLOCATABLE_FREG_NUMS[color as usize]);
                RvVarLocation::FReg(freg)
            } else {
                let offset = (LOCATION_SIZE as i32) * (color - ALLOCATABLE_XREGS_SIZE + 1) as i32;
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
    // for (k, v) in &x_var_color {
    //     let color = *v as u8;
    //     if color < ALLOCATABLE_XREGS_SIZE {
    //         println!("{} -> {}", k, RvVarLocation::XReg(XReg::from_u8(ALLOCATABLE_XREG_NUMS[color as usize])));
    //     } else {
    //         println!(
    //             "{} -> {}",
    //             k,
    //             RvVarLocation::StackSlot {
    //                 offset: ((color - ALLOCATABLE_XREGS_SIZE + 1) * LOCATION_SIZE) as i32,
    //                 size: 8
    //             }
    //         );
    //     }
    // }
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

//
// /// 正确的贪心图着色：物理寄存器预着色为自身编号，变量按度数降序取最小空闲可分配编号，
// /// 无号可取时标记为溢出。
// fn color_graph(
//     graph: &RvVarLocationGraph,
//     allocatable: &[u8],
// ) -> (HashMap<RvVarLocation, Color>, HashSet<RvVarLocation>) {
//     let mut colors = HashMap::new();
//     let mut spilled = HashSet::new();
//
//     for location in graph.all_locations() {
//         match location {
//             RvVarLocation::XReg(reg) => {
//                 colors.insert(location, reg.num());
//             },
//             RvVarLocation::FReg(reg) => {
//                 colors.insert(location, reg.num());
//             },
//             _ => {},
//         }
//     }
//
//     let mut variables: Vec<RvVarLocation> = graph
//         .all_locations()
//         .into_iter()
//         .filter(|location| matches!(location, RvVarLocation::XVar(_) | RvVarLocation::FVar(_)))
//         .collect();
//     variables.sort_by_key(|location| std::cmp::Reverse(graph.neighbors(location).len()));
//
//     for variable in variables {
//         let mut forbidden = HashSet::new();
//         for neighbor in graph.neighbors(&variable) {
//             if let Some(color) = colors.get(&neighbor) {
//                 forbidden.insert(*color);
//             }
//         }
//         match allocatable.iter().find(|&&color| !forbidden.contains(&color)) {
//             Some(&color) => {
//                 colors.insert(variable, color);
//             },
//             None => {
//                 spilled.insert(variable);
//             },
//         }
//     }
//
//     (colors, spilled)
// }
//
// /// 把虚拟位置替换为物理寄存器（颜色即寄存器编号）。
// fn location_to_reg(
//     location: &RvVarLocation,
//     x_color_map: &HashMap<RvVarLocation, Color>,
//     f_color_map: &HashMap<RvVarLocation, Color>,
// ) -> RvVarLocation {
//     match location {
//         RvVarLocation::XVar(_) => RvVarLocation::XReg(XReg::from_u8(x_color_map[location])),
//         RvVarLocation::FVar(_) => RvVarLocation::FReg(FReg::from_u8(f_color_map[location])),
//         _ => location.clone(),
//     }
// }
//
// fn replace_locations(
//     basic_block: &RvVarBasicBlock,
//     x_color_map: &HashMap<RvVarLocation, Color>,
//     f_color_map: &HashMap<RvVarLocation, Color>,
// ) -> RvVarBasicBlock {
//     let mut instrs = Vec::new();
//     for instr in &basic_block.instrs {
//         let mut map_dest = |location: &RvVarLocation| location_to_reg(location, x_color_map, f_color_map);
//         let mut map_src = |location: &RvVarLocation| location_to_reg(location, x_color_map, f_color_map);
//         instrs.push(instr.map_operands(&mut map_dest, &mut map_src));
//     }
//     RvVarBasicBlock { name: basic_block.name.clone(), instrs }
// }
//
// /// 溢出时的栈槽与临时变量分配状态，跨基本块与迭代复用同一变量的槽位。
// struct SpillState {
//     offsets: HashMap<RvVarLocation, i16>,
//     next_offset: i16,
//     next_temp: u64,
// }
//
// impl SpillState {
//     fn new() -> Self {
//         SpillState { offsets: HashMap::new(), next_offset: -8, next_temp: 0 }
//     }
//
//     fn offset_of(&mut self, location: &RvVarLocation) -> i16 {
//         if let Some(&offset) = self.offsets.get(location) {
//             offset
//         } else {
//             let offset = self.next_offset;
//             self.next_offset -= 8;
//             self.offsets.insert(location.clone(), offset);
//             offset
//         }
//     }
//
//     fn fresh(&mut self, float: bool) -> RvVarLocation {
//         let name = format!("$spill{}", self.next_temp);
//         self.next_temp += 1;
//         if float {
//             RvVarLocation::FVar(name)
//         } else {
//             RvVarLocation::XVar(name)
//         }
//     }
// }
//
// /// 溢出改写：溢出的源前插 `ld/fld`，溢出的目标后插 `sd/fsd`，操作数替换为新临时变量。
// fn rewrite_spills(
//     block: &RvVarBasicBlock,
//     x_spilled: &HashSet<RvVarLocation>,
//     f_spilled: &HashSet<RvVarLocation>,
//     state: &RefCell<SpillState>,
// ) -> RvVarBasicBlock {
//     let mut instrs = Vec::new();
//     for instr in &block.instrs {
//         let mut loads: Vec<RvVarInstr> = Vec::new();
//         let mut stores: Vec<RvVarInstr> = Vec::new();
//
//         let new_instr = {
//             let mut map_dest = |location: &RvVarLocation| {
//                 if x_spilled.contains(location) {
//                     let mut state = state.borrow_mut();
//                     let tmp = state.fresh(false);
//                     let offset = state.offset_of(location);
//                     stores.push(RvVarInstr::Sd { rs2: tmp.clone(), rs1: sp(), imm: Imm12::from_i16(offset) });
//                     tmp
//                 } else if f_spilled.contains(location) {
//                     let mut state = state.borrow_mut();
//                     let tmp = state.fresh(true);
//                     let offset = state.offset_of(location);
//                     stores.push(RvVarInstr::Fsd { rs2: tmp.clone(), rs1: sp(), imm: Imm12::from_i16(offset) });
//                     tmp
//                 } else {
//                     location.clone()
//                 }
//             };
//             let mut map_src = |location: &RvVarLocation| {
//                 if x_spilled.contains(location) {
//                     let mut state = state.borrow_mut();
//                     let tmp = state.fresh(false);
//                     let offset = state.offset_of(location);
//                     loads.push(RvVarInstr::Ld { rd: tmp.clone(), rs1: sp(), imm: Imm12::from_i16(offset) });
//                     tmp
//                 } else if f_spilled.contains(location) {
//                     let mut state = state.borrow_mut();
//                     let tmp = state.fresh(true);
//                     let offset = state.offset_of(location);
//                     loads.push(RvVarInstr::Fld { rd: tmp.clone(), rs1: sp(), imm: Imm12::from_i16(offset) });
//                     tmp
//                 } else {
//                     location.clone()
//                 }
//             };
//             instr.map_operands(&mut map_dest, &mut map_src)
//         };
//
//         instrs.extend(loads);
//         instrs.push(new_instr);
//         instrs.extend(stores);
//     }
//     RvVarBasicBlock { name: block.name.clone(), instrs }
// }
//
// fn allocate_block(block: &RvVarBasicBlock, state: &RefCell<SpillState>) -> RvVarBasicBlock {
//     let mut current = block.clone();
//     for _ in 0..64 {
//         let liveness = liveness_analysis(&current);
//         let (x_graph, f_graph) = build_infer_graph(&liveness);
//         let (x_color, x_spilled) = color_graph(&x_graph, &ALLOCATABLE_XREG_NUMS);
//         let (f_color, f_spilled) = color_graph(&f_graph, &ALLOCATABLE_FREG_NUMS);
//         if x_spilled.is_empty() && f_spilled.is_empty() {
//             return replace_locations(&current, &x_color, &f_color);
//         }
//         current = rewrite_spills(&current, &x_spilled, &f_spilled, state);
//     }
//     panic!("register allocation did not converge");
// }

// pub fn allocate_registers(var_prog: &RvVarProgram) -> RvVarProgram {
//     let mut prog = RvVarProgram::new();
//     let state = RefCell::new(SpillState::new());
//     for basic_block in &var_prog.blocks {
//         prog.append_basic_block(allocate_block(basic_block, &state));
//     }
//     prog
// }

// =============================================================================
// 以下是旧的饱和度着色（saturation-based coloring）实现，已注释保留供参考。
//
// 依赖的导入（均已从文件头部移除）：
//     use crate::riscv_var::location::x;
//     use priority_queue::PriorityQueue;
//     use std::cell::Cell;
//     use std::collections::BTreeSet;
//     use std::hash::{Hash, Hasher};
//
// 该实现的问题在于：`color_graph` 里 `init` 之后 clone 了一份优先队列，
// 但 `set_color` / `add_staturation` 只更新 `location_staturation_map` 里的对象，
// 队列里的 `LocationStaturation` 是过期快照，染色时读到的 saturation 不含
// 后来着色邻居的颜色，因此可能给两个相互干涉的变量分配同一个颜色。
// =============================================================================
/*
trait InterfereGraph {
    fn interfere(&self, location1: &RvVarLocation, location2: &RvVarLocation);
    fn add_location(&mut self, location: &RvVarLocation) -> NodeIndex;
}

fn allocatable_xregs() -> HashSet<RvVarLocation> {
    let regs = vec![
        x(4), x(5), x(6), x(7), x(9), x(10), x(11), x(12), x(13), x(14), x(15), x(16), x(17),
        x(18), x(19), x(20), x(21), x(22), x(23), x(24), x(25), x(26), x(27), x(28), x(29),
        x(30), x(31),
    ];
    regs.into_iter().collect()
}

//  ZERO(x0): 恒为 0。 RA(x1): 返回地址。SP(x2): 栈指针。GP(x3): 全局指针。FP(x8): 帧指针。TP(x4):线程指针。
fn non_allocatable_xregs() -> HashSet<RvVarLocation> {
    let regs = vec![x(0), x(1), x(2), x(3), x(4), x(8)];
    regs.into_iter().collect()
}

fn non_allocatable_xregs_color(xreg: &XReg) -> Color {
    todo!()
}

type Color = i8;
const UNCOLORED: i8 = 0;

#[derive(Clone)]
struct LocationStaturation {
    staturation: BTreeSet<Color>,
    color: Cell<Color>,
    location: RvVarLocation,
}

impl PartialEq for LocationStaturation {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}
impl Eq for LocationStaturation {}

impl Hash for LocationStaturation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
    }
}

impl LocationStaturation {
    fn new(location: RvVarLocation) -> Self {
        let color = Cell::new(UNCOLORED);
        let staturation = BTreeSet::new();
        LocationStaturation { color, staturation, location }
    }
}

// impl PartialOrd for LocationStaturation {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }
//
// impl Ord for LocationStaturation {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.staturation.len().cmp(&other.staturation.len())
//     }
// }
struct RvVarLocationStaturationGraph {
    location_graph: RvVarLocationGraph,
    location_staturation_map: HashMap<RvVarLocation, LocationStaturation>,
    location_staturation_queue: PriorityQueue<LocationStaturation, usize>,
}

impl RvVarLocationStaturationGraph {
    pub fn new() -> Self {
        let location_graph = RvVarLocationGraph::new();
        let location_staturation_map = HashMap::new();
        let location_staturation_queue = PriorityQueue::new();
        RvVarLocationStaturationGraph { location_graph, location_staturation_map, location_staturation_queue }
    }

    fn push_location_staturation_to_queue(&mut self, location_staturation: LocationStaturation) {
        let color = location_staturation.color.get();
        if color == UNCOLORED {
            let priority = location_staturation.staturation.len();
            self.location_staturation_queue.push(location_staturation, priority);
        }
    }

    fn add_location(&mut self, location: &RvVarLocation) -> NodeIndex {
        let idx = self.location_graph.add_location(location);
        let location_staturation = LocationStaturation::new(location.clone());
        self.location_staturation_map.insert(location.clone(), location_staturation.clone());
        self.push_location_staturation_to_queue(location_staturation);
        idx
    }

    fn interfere(&mut self, location1: &RvVarLocation, location2: &RvVarLocation) {
        self.location_graph.interfere(location1, location2);
        let color1 = self.location_staturation_map.get(location1).unwrap().color.get();
        let color2 = self.location_staturation_map.get(location2).unwrap().color.get();
        if color1 != UNCOLORED {
            self.add_staturation(location2, color1);
        }
        if color2 != UNCOLORED {
            self.add_staturation(location1, color2);
        }

        if let Some(location_staturation) = self.location_staturation_map.get(location1) {
            self.push_location_staturation_to_queue(location_staturation.clone());
        }
        if let Some(location_staturation) = self.location_staturation_map.get(location2) {
            self.push_location_staturation_to_queue(location_staturation.clone());
        }
    }

    pub fn init(&mut self, location_graph: &RvVarLocationGraph) {
        // 初始化不能分配的x寄存器的saturation和color
        let non_allocatable = non_allocatable_xregs();
        let non_allocatable_xregs_color = non_allocatable_xregs_color();

        let all_locations = location_graph.all_locations();
        for location in &all_locations {
            self.add_location(&location);
            if non_allocatable.contains(&location) {
                if let Some(location_staturation) = self.location_staturation_map.get_mut(location) {
                    let color = *non_allocatable_xregs_color.get(&location).unwrap();
                    location_staturation.color.set(color);
                }
            };
        }

        for location in all_locations {
            for neighbor in location_graph.neighbors(&location) {
                self.interfere(&location, &neighbor);
            }
        }
    }

    fn add_staturation(&mut self, location: &RvVarLocation, color: Color) {
        if let Some(location_staturation) = self.location_staturation_map.get_mut(location) {
            assert!(color != UNCOLORED);
            location_staturation.staturation.insert(color);
        }
    }

    fn set_color(&mut self, location: &RvVarLocation, color: Color) {
        if let Some(location_staturation) = self.location_staturation_map.get_mut(location) {
            location_staturation.color.set(color);
            for neighbor in self.location_graph.neighbors(location) {
                self.add_staturation(&neighbor, color);
            }
        }
    }
}

fn non_allocatable_xregs_color() -> HashMap<RvVarLocation, Color> {
    let mut color_map = HashMap::<RvVarLocation, Color>::new();
    let mut i = -1;
    for location in non_allocatable_xregs() {
        color_map.insert(location, i);
        i -= 1;
    }
    color_map
}

fn color_graph(graph: &RvVarLocationGraph) -> HashMap<RvVarLocation, Color> {
    let mut location_staturation_graph = RvVarLocationStaturationGraph::new();
    location_staturation_graph.init(graph);
    let mut queue = location_staturation_graph.location_staturation_queue.clone();
    while !queue.is_empty() {
        if let Some((location_staturation, _)) = queue.pop() {
            assert!(location_staturation.color.get() == UNCOLORED);

            let mut location_color = 1;
            for staturation in location_staturation.staturation.iter() {
                if *staturation <= 0 {
                    continue;
                }
                if *staturation == location_color {
                    location_color += 1;
                } else {
                    break;
                }
            }

            location_staturation_graph.set_color(&location_staturation.location, location_color);
        }
    }

    let mut color_map = HashMap::<RvVarLocation, Color>::new();
    for (location, location_staturation) in location_staturation_graph.location_staturation_map {
        color_map.insert(location, location_staturation.color.get());
    }
    color_map
}

fn x_color_to_xreg(color: Color) -> XReg {
    assert!(color > 0, "x_color_to_xreg: expected positive color, got {color}");
    XReg::from_u8(ALLOCATABLE_XREG_NUMS[(color - 1) as usize])
}

fn f_color_to_xreg(color: Color) -> FReg {
    assert!(color > 0, "f_color_to_xreg: expected positive color, got {color}");
    FReg::from_u8((color - 1) as u8)
}

// x寄存器
fn x_location_to_xreg(location: &RvVarLocation, x_color_map: &HashMap<RvVarLocation, Color>) -> RvVarLocation {
    match location {
        RvVarLocation::XReg(_) => location.clone(),
        RvVarLocation::XVar(_) => {
            let color = x_color_map[location];
            RvVarLocation::XReg(x_color_to_xreg(color))
        },
        _ => panic!("x_location_to_xreg: unexpected location {location}"),
    }
}

// 浮点数寄存器
fn f_location_to_freg(location: &RvVarLocation, f_color_map: &HashMap<RvVarLocation, Color>) -> RvVarLocation {
    match location {
        RvVarLocation::FReg(_) => location.clone(),
        RvVarLocation::FVar(_) => {
            let color = f_color_map[location];
            RvVarLocation::FReg(f_color_to_freg(color))
        },
        _ => panic!("f_location_to_freg: unexpected location {location}"),
    }
}
*/

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
