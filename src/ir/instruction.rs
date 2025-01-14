//======---------------------------------------------------------------======//
//                                                                           //
// Copyright 2022 Evan Cox <evanacox00@gmail.com>. All rights reserved.      //
//                                                                           //
// Use of this source code is governed by a BSD-style license that can be    //
// found in the LICENSE.txt file at the root of this project, or at the      //
// following link: https://opensource.org/licenses/BSD-3-Clause              //
//                                                                           //
//======---------------------------------------------------------------======//

use crate::arena::ArenaKey;
use crate::ir::{Block, FloatFormat, Func, Sig, Type, Value};
use crate::utility::{PackedOption, TinyArray};
use smallvec::SmallVec;
use static_assertions::assert_eq_size;
use std::{iter, slice};

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// This holds both the opcode of a given instruction and all the state
/// that makes up that specific instruction.
///
/// While each instruction may have wildly different actual data, they all
/// are stored in the same table and all inside the same `InstructionData` type.
#[repr(u32)]
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum InstData {
    /// `call T @function(args...)`, models a direct call to a known function.
    Call(CallInst),
    /// `call T %var(args...)`, models an indirect call through a pointer.
    IndirectCall(IndirectCallInst),
    /// `icmp op T %a, %b`, models an integer comparison
    ICmp(ICmpInst),
    /// `fcmp op T %a, %b`, models a floating-point comparison
    FCmp(FCmpInst),
    /// `sel bool %cond, T %a, T %b`, models a ternary-like instruction
    Sel(SelInst),
    /// `br block`, models an unconditional branch
    Br(BrInst),
    /// `condbr bool %cond, if block1, else block2`, models a conditional branch between two blocks
    CondBr(CondBrInst),
    /// `unreachable`, a terminator that can never be executed
    Unreachable(UnreachableInst),
    /// `ret T %val`, returns from the current function
    Ret(RetInst),
    /// `and T %a, %b`, performs bitwise AND
    And(CommutativeArithInst),
    /// `or T %a, %b`, performs bitwise OR
    Or(CommutativeArithInst),
    /// `xor T %a, %b`, performs bitwise XOR
    Xor(CommutativeArithInst),
    /// `shl T %a, %b`, performs bitwise left-shift
    Shl(ArithInst),
    /// `ashr T %a, %b`, performs bitwise arithmetic right-shift
    AShr(ArithInst),
    /// `lshr T %a, %b`, performs bitwise logical right-shift
    LShr(ArithInst),
    /// `iadd T %a, %b`, performs two's complement addition
    IAdd(CommutativeArithInst),
    /// `isub T %a, %b`, performs two's complement subtraction
    ISub(ArithInst),
    /// `imul T %a, %b`, performs two's complement multiplication
    IMul(CommutativeArithInst),
    /// `sdiv T %a, %b`, performs signed division
    SDiv(ArithInst),
    /// `udiv T %a, %b`, performs unsigned division
    UDiv(ArithInst),
    /// `srem T %a, %b`, gets the remainder of performing signed division
    SRem(ArithInst),
    /// `urem T %a, %b`, gets the remainder of performing unsigned division
    URem(ArithInst),
    /// `fneg T %a`, performs floating-point negation
    FNeg(FloatUnaryInst),
    /// `fadd T %a, %b`, performs floating-point addition
    FAdd(ArithInst),
    /// `fadd T %a, %b`, performs floating-point subtraction
    FSub(ArithInst),
    /// `fadd T %a, %b`, performs floating-point multiplication
    FMul(ArithInst),
    /// `fadd T %a, %b`, performs floating-point division
    FDiv(ArithInst),
    /// `fadd T %a, %b`, gets the remainder of performing floating-point division
    FRem(ArithInst),
    /// `alloca T`, performs stack allocation
    Alloca(AllocaInst),
    /// `load T, ptr %p`, loads from a pointer
    Load(LoadInst),
    /// `store T %a, ptr %p`, stores a value to a pointer
    Store(StoreInst),
    /// `offset T, ptr %p`, performs pointer arithmetic
    Offset(OffsetInst),
    /// `extract T %s, N`, extracts a value from an aggregate
    Extract(ExtractInst),
    /// `insert T %s, U %a, N`, inserts a value into an aggregate
    Insert(InsertInst),
    /// `elemptr T, ptr %p, N`, gets a pointer into an aggregate
    ElemPtr(ElemPtrInst),
    /// `sext T, U %b`, performs sign-extension
    Sext(CastInst),
    /// `zext T, U %b`, performs zero-extension
    Zext(CastInst),
    /// `trunc T, U %b`, performs truncation
    Trunc(CastInst),
    /// `itob T, U %b`, converts an integer into a `bool`
    IToB(CastInst),
    /// `btoi T, U %b`, converts a `bool` into an integer
    BToI(CastInst),
    /// `sitof T, U %b`, converts a signed integer into a float
    SIToF(CastInst),
    /// `itob T, U %b`, converts an unsigned integer into a float
    UIToF(CastInst),
    /// `ftosi T, U %b`, converts a float into a signed integer
    FToSI(CastInst),
    /// `ftoui T, U %b`, converts a float into an unsigned integer
    FToUI(CastInst),
    /// `fext T, U %b`, converts a float into a larger float
    FExt(CastInst),
    /// `ftrunc T, U %b`, converts a float into a smaller float
    FTrunc(CastInst),
    /// `itop T, U %b`, converts an integer into a pointer
    IToP(CastInst),
    /// `ptoi T, U %b`, converts a pointer into an integer
    PToI(CastInst),
    /// `iconst T N`, materializes an integer constant
    IConst(IConstInst),
    /// `fconst T N`, materializes a floating-point constant
    FConst(FConstInst),
    /// `bconst N`, materializes a bool constant
    BConst(BConstInst),
    /// `undef T N`, materializes an uninitialized value
    Undef(UndefConstInst),
    /// `null T N`, materializes a null value
    Null(NullConstInst),
    /// `globaladdr @name`, materializes a pointer to a global value
    GlobalAddr(GlobalAddrInst),
}

impl Instruction for InstData {
    fn operands(&self) -> &[Value] {
        match self {
            InstData::Call(e) => e.operands(),
            InstData::IndirectCall(e) => e.operands(),
            InstData::ICmp(e) => e.operands(),
            InstData::FCmp(e) => e.operands(),
            InstData::Sel(e) => e.operands(),
            InstData::Br(e) => e.operands(),
            InstData::CondBr(e) => e.operands(),
            InstData::Unreachable(e) => e.operands(),
            InstData::Ret(e) => e.operands(),
            InstData::And(e) => e.operands(),
            InstData::Or(e) => e.operands(),
            InstData::Xor(e) => e.operands(),
            InstData::Shl(e) => e.operands(),
            InstData::AShr(e) => e.operands(),
            InstData::LShr(e) => e.operands(),
            InstData::IAdd(e) => e.operands(),
            InstData::ISub(e) => e.operands(),
            InstData::IMul(e) => e.operands(),
            InstData::SDiv(e) => e.operands(),
            InstData::UDiv(e) => e.operands(),
            InstData::SRem(e) => e.operands(),
            InstData::URem(e) => e.operands(),
            InstData::FNeg(e) => e.operands(),
            InstData::FAdd(e) => e.operands(),
            InstData::FSub(e) => e.operands(),
            InstData::FMul(e) => e.operands(),
            InstData::FDiv(e) => e.operands(),
            InstData::FRem(e) => e.operands(),
            InstData::Alloca(e) => e.operands(),
            InstData::Load(e) => e.operands(),
            InstData::Store(e) => e.operands(),
            InstData::Offset(e) => e.operands(),
            InstData::Extract(e) => e.operands(),
            InstData::Insert(e) => e.operands(),
            InstData::ElemPtr(e) => e.operands(),
            InstData::Sext(e) => e.operands(),
            InstData::Zext(e) => e.operands(),
            InstData::Trunc(e) => e.operands(),
            InstData::IToB(e) => e.operands(),
            InstData::BToI(e) => e.operands(),
            InstData::SIToF(e) => e.operands(),
            InstData::UIToF(e) => e.operands(),
            InstData::FToSI(e) => e.operands(),
            InstData::FToUI(e) => e.operands(),
            InstData::FExt(e) => e.operands(),
            InstData::FTrunc(e) => e.operands(),
            InstData::IToP(e) => e.operands(),
            InstData::PToI(e) => e.operands(),
            InstData::IConst(e) => e.operands(),
            InstData::FConst(e) => e.operands(),
            InstData::BConst(e) => e.operands(),
            InstData::Undef(e) => e.operands(),
            InstData::Null(e) => e.operands(),
            InstData::GlobalAddr(e) => e.operands(),
        }
    }

    fn result_ty(&self) -> Option<Type> {
        match self {
            InstData::Call(e) => e.result_ty(),
            InstData::IndirectCall(e) => e.result_ty(),
            InstData::ICmp(e) => e.result_ty(),
            InstData::FCmp(e) => e.result_ty(),
            InstData::Sel(e) => e.result_ty(),
            InstData::Br(e) => e.result_ty(),
            InstData::CondBr(e) => e.result_ty(),
            InstData::Unreachable(e) => e.result_ty(),
            InstData::Ret(e) => e.result_ty(),
            InstData::And(e) => e.result_ty(),
            InstData::Or(e) => e.result_ty(),
            InstData::Xor(e) => e.result_ty(),
            InstData::Shl(e) => e.result_ty(),
            InstData::AShr(e) => e.result_ty(),
            InstData::LShr(e) => e.result_ty(),
            InstData::IAdd(e) => e.result_ty(),
            InstData::ISub(e) => e.result_ty(),
            InstData::IMul(e) => e.result_ty(),
            InstData::SDiv(e) => e.result_ty(),
            InstData::UDiv(e) => e.result_ty(),
            InstData::SRem(e) => e.result_ty(),
            InstData::URem(e) => e.result_ty(),
            InstData::FNeg(e) => e.result_ty(),
            InstData::FAdd(e) => e.result_ty(),
            InstData::FSub(e) => e.result_ty(),
            InstData::FMul(e) => e.result_ty(),
            InstData::FDiv(e) => e.result_ty(),
            InstData::FRem(e) => e.result_ty(),
            InstData::Alloca(e) => e.result_ty(),
            InstData::Load(e) => e.result_ty(),
            InstData::Store(e) => e.result_ty(),
            InstData::Offset(e) => e.result_ty(),
            InstData::Extract(e) => e.result_ty(),
            InstData::Insert(e) => e.result_ty(),
            InstData::ElemPtr(e) => e.result_ty(),
            InstData::Sext(e) => e.result_ty(),
            InstData::Zext(e) => e.result_ty(),
            InstData::Trunc(e) => e.result_ty(),
            InstData::IToB(e) => e.result_ty(),
            InstData::BToI(e) => e.result_ty(),
            InstData::SIToF(e) => e.result_ty(),
            InstData::UIToF(e) => e.result_ty(),
            InstData::FToSI(e) => e.result_ty(),
            InstData::FToUI(e) => e.result_ty(),
            InstData::FExt(e) => e.result_ty(),
            InstData::FTrunc(e) => e.result_ty(),
            InstData::IToP(e) => e.result_ty(),
            InstData::PToI(e) => e.result_ty(),
            InstData::IConst(e) => e.result_ty(),
            InstData::FConst(e) => e.result_ty(),
            InstData::BConst(e) => e.result_ty(),
            InstData::Undef(e) => e.result_ty(),
            InstData::Null(e) => e.result_ty(),
            InstData::GlobalAddr(e) => e.result_ty(),
        }
    }
}

/// These are the properties that any transform or analysis pass needs to be
/// able to observe for any given instruction in any block.
///
/// Any instruction's data can be passed as a `&dyn Instruction`, because every
/// valid opcode has at least an implementation for `Instruction`.
pub trait Instruction {
    /// Gets any operands that the instruction operates on.
    ///
    /// Note that this may be an empty array, it is not safe to assume that
    /// there will be at least one operand.
    fn operands(&self) -> &[Value];

    /// Gets the type of the instruction's result after it has been evaluated.  
    ///
    /// Not all instructions will have one of these, e.g. terminators, `call void`s
    /// and `store`s do not evaluate to anything.
    fn result_ty(&self) -> Option<Type>;

    /// Checks if the instruction yields any results at all.
    fn has_result(&self) -> bool {
        self.result_ty().is_none()
    }
}

/// Some instructions model binary operations, those instructions model this trait.
///
/// A valid assumption for any type implementing this trait is that the operands
/// returned by [`Instruction::operands`] has a length of exactly 2. However,
/// it is not valid to assume that any instruction implementing this follows the
/// same pattern as most arithmetic instructions (i.e. [`Self::lhs`] and [`Self::rhs`] do **not**
/// always have the same type, and [`Instruction::result_ty`] may be different from
/// the type of the operands).
pub trait BinaryInst: Instruction {
    /// Gets the left-hand operand of the instruction. For `inst T %a, %b`,
    /// this would be `%a`.
    fn lhs(&self) -> Value {
        self.operands()[0]
    }

    /// Gets the left-hand operand of the instruction. For `inst T %a, %b`,
    /// this would be `%b`.
    fn rhs(&self) -> Value {
        self.operands()[1]
    }

    /// Checks if the binary instruction follows the commutative property, i.e.
    /// whether it is safe to swap the operands while preserving the behavior.
    fn is_commutative(&self) -> bool;
}

/// Some instructions model unary operations, those instructions model this trait.
///
/// A valid assumption for any type implementing this trait is that the operands
/// returned by [`Instruction::operands`] has a length of exactly 1.
///
/// While most instructions implementing this are casts, not all of them are.
/// A few instructions (notably `fneg`) are unary but not casts.
pub trait UnaryInst: Instruction {
    /// Gets the single unary operand of the instruction. For `inst T %a`, this
    /// would be `%a`.
    fn operand(&self) -> Value {
        self.operands()[0]
    }
}

/// Models a branch target, along with any arguments being passed into that block.
///
/// The argument info is just as important as the branch target itself, and the two are
/// tied together at any given usage. Thus, both are always stored.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct BlockWithParams {
    // stores both the block and any params, params is usually length 0 so this works
    // fine without needing a heap allocation
    data: TinyArray<Value, 2>,
}

impl BlockWithParams {
    /// Creates a branch target that includes an optional list of arguments.
    ///
    /// An empty `params` list signifies no arguments being passed.  
    pub fn new(target: Block, params: &[Value]) -> Self {
        let mut vals = SmallVec::<[Value; 2]>::new();

        vals.push(Value::raw_from(target));
        vals.extend_from_slice(params);

        Self {
            data: TinyArray::from_small_vec(vals),
        }
    }

    /// Same as [`Self::new`] but specifically when there's a pre-existing vec
    /// of parameters.
    ///
    /// An empty `params` list signifies no arguments being passed.  
    pub fn from_vec(target: Block, mut params: Vec<Value>) -> Self {
        params.insert(0, Value::raw_from(target));

        Self {
            data: TinyArray::from_vec(params),
        }
    }

    /// Gets the block target by itself.
    #[inline]
    pub fn block(&self) -> Block {
        Block::new(self.data[0].index())
    }

    /// Gets the block arguments being passed, if any
    #[inline]
    pub fn args(&self) -> &[Value] {
        match self.data.get(1..) {
            Some(vals) => vals,
            None => &[],
        }
    }
}

/// Models a terminator, i.e. the only instructions that are allowed at the end
/// of a basic block.
///
/// All terminators transfer control flow *somewhere* unless they end execution,
/// so users need to be able to query where control could be transferred to.
pub trait Terminator: Instruction {
    /// Gets the possible blocks where control could be transferred to
    /// once this instruction is executed.
    ///
    /// Note that this might be empty, see `unreachable` or `ret`.
    fn targets(&self) -> &[BlockWithParams];

    #[doc(hidden)]
    fn __operands(&self) -> &[Value];
}

impl<T: Terminator> Instruction for T {
    fn operands(&self) -> &[Value] {
        self.__operands()
    }

    fn result_ty(&self) -> Option<Type> {
        None
    }
}

/// Models the different ways that integers values can be compared in SIR
/// using the `icmp` instruction.
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum ICmpOp {
    /// `eq`, checks if the integers are (bitwise) equivalent
    EQ,
    /// `ne`, checks if the integers are (bitwise) not-equal
    NE,
    /// `sgt`, treats both integers as signed and checks if `a > b`
    SGT,
    /// `slt`, treats both integers as signed and checks if `a < b`
    SLT,
    /// `sge`, treats both integers as signed and checks if `a >= b`
    SGE,
    /// `sle`, treats both integers as signed and checks if `a <= b`
    SLE,
    /// `ugt`, treats both integers as unsigned and checks if `a > b`
    UGT,
    /// `ult`, treats both integers as unsigned and checks if `a < b`
    ULT,
    /// `uge`, treats both integers as unsigned and checks if `a >= b`
    UGE,
    /// `ule`, treats both integers as unsigned and checks if `a <= b`
    ULE,
}

/// Models a single `icmp` instruction.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct ICmpInst {
    comparison: ICmpOp,
    operands: [Value; 2],
}

impl ICmpInst {
    pub(in crate::ir) fn new(cmp: ICmpOp, lhs: Value, rhs: Value) -> Self {
        Self {
            comparison: cmp,
            operands: [lhs, rhs],
        }
    }

    /// Gets the comparison that the `icmp` is performing between
    /// the two operands.
    pub fn op(&self) -> ICmpOp {
        self.comparison
    }
}

impl Instruction for ICmpInst {
    fn operands(&self) -> &[Value] {
        &self.operands
    }

    fn result_ty(&self) -> Option<Type> {
        Some(Type::bool())
    }
}

impl BinaryInst for ICmpInst {
    fn is_commutative(&self) -> bool {
        matches!(self.op(), ICmpOp::EQ | ICmpOp::NE)
    }
}

/// Models the different ways that floating-point values can be
/// compared in SIR.
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum FCmpOp {
    /// `uno`, "unordered." Checks if either operand is a `NaN`.
    UNO,
    /// `ord`, "ordered." Checks that both operands are not `NaN`s.
    ORD,
    /// `oeq`, "ordered and equal." Checks if the operands are ordered and `a == b`.
    OEQ,
    /// `one`, "ordered and not equal." Checks if the operands are ordered and `a != b`
    ONE,
    /// `ogt`, "ordered and greater-than." Checks if both operands are ordered and that `a > b`
    OGT,
    /// `olt`, "ordered and less-than." Checks if both operands are ordered and that `a < b`
    OLT,
    /// `oge`, "ordered and greater-than-or-equals." Checks if both operands are ordered and `a >= b`
    OGE,
    /// `ole`, "ordered and less-than-or-equals." Checks if both operands are ordered and `a <= b`
    OLE,
    /// `ueq`, "unordered and equal." Checks if the operands are unordered and `a == b`.
    UEQ,
    /// `une`, "unordered and not equal." Checks if the operands are unordered and `a != b`
    UNE,
    /// `ugt`, "unordered and greater-than." Checks if both operands are unordered and that `a > b`
    UGT,
    /// `olt`, "unordered and less-than." Checks if both operands are unordered and that `a < b`
    ULT,
    /// `uge`, "unordered and greater-than-or-equals." Checks if both operands are unordered and `a >= b`
    UGE,
    /// `ule`, "unordered and less-than-or-equals." Checks if both operands are unordered and `a <= b`
    ULE,
}

/// Models an `fcmp` instruction.
///
/// ```raw
/// %2 = fcmp oeq f64 %0, %1
/// ```
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct FCmpInst {
    comparison: FCmpOp,
    operands: [Value; 2],
}

impl FCmpInst {
    pub(in crate::ir) fn new(comp: FCmpOp, lhs: Value, rhs: Value) -> Self {
        Self {
            comparison: comp,
            operands: [lhs, rhs],
        }
    }

    /// Gets the comparison the `fcmp` is performing between the two operands.
    pub fn op(&self) -> FCmpOp {
        self.comparison
    }
}

impl Instruction for FCmpInst {
    fn operands(&self) -> &[Value] {
        &self.operands
    }

    fn result_ty(&self) -> Option<Type> {
        Some(Type::bool())
    }
}

impl BinaryInst for FCmpInst {
    fn is_commutative(&self) -> bool {
        matches!(
            self.op(),
            FCmpOp::OEQ | FCmpOp::ONE | FCmpOp::UEQ | FCmpOp::UNE
        )
    }
}

/// Models a `sel` instruction.
///
/// ```raw
/// %3 = sel T, bool %0, %1, %2
/// ```
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct SelInst {
    operands: [Value; 3],
    output: Type,
}

assert_eq_size!(SelInst, [u64; 2]);

impl SelInst {
    pub(in crate::ir) fn new(output: Type, cond: Value, if_true: Value, otherwise: Value) -> Self {
        Self {
            output,
            operands: [cond, if_true, otherwise],
        }
    }

    /// Gets the condition that determines which value is chosen.
    pub fn condition(&self) -> Value {
        self.operands[0]
    }

    /// Gets the value yielded if [`Self::condition`] is `true`.
    pub fn if_true(&self) -> Value {
        self.operands[1]
    }

    /// Gets the value yielded if [`Self::condition`] is `false`.
    pub fn if_false(&self) -> Value {
        self.operands[2]
    }
}

impl Instruction for SelInst {
    fn operands(&self) -> &[Value] {
        &self.operands
    }

    fn result_ty(&self) -> Option<Type> {
        Some(self.output)
    }
}

/// Models a `br` instruction
///
/// ```raw
/// br block
/// ```
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct BrInst {
    target: BlockWithParams,
}

impl BrInst {
    pub(in crate::ir) fn new(target: BlockWithParams) -> Self {
        Self { target }
    }

    /// Gets the target branch being jumped to
    pub fn target(&self) -> &BlockWithParams {
        &self.target
    }
}

impl Terminator for BrInst {
    fn targets(&self) -> &[BlockWithParams] {
        slice::from_ref(&self.target)
    }

    fn __operands(&self) -> &[Value] {
        self.target.args()
    }
}

/// Models a conditional branch
///
/// ```raw
/// condbr bool %0, block1, block2
/// ```
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct CondBrInst {
    condition: Value,
    targets: [BlockWithParams; 2],
}

impl CondBrInst {
    pub(in crate::ir) fn new(
        cond: Value,
        if_true: BlockWithParams,
        otherwise: BlockWithParams,
    ) -> Self {
        Self {
            condition: cond,
            targets: [if_true, otherwise],
        }
    }

    /// Gets the condition being checked in the `condbr`
    pub fn condition(&self) -> Value {
        self.condition
    }

    /// Gets the branch being jumped to if the condition is `true`
    pub fn true_branch(&self) -> &BlockWithParams {
        &self.targets[0]
    }

    /// Gets the branch being jumped to if the condition is `false`
    pub fn false_branch(&self) -> &BlockWithParams {
        &self.targets[1]
    }
}

impl Terminator for CondBrInst {
    fn targets(&self) -> &[BlockWithParams] {
        &self.targets
    }

    fn __operands(&self) -> &[Value] {
        slice::from_ref(&self.condition)
    }
}

/// Gets an `unreachable` instruction
///
/// ```raw
/// unreachable
/// ```
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct UnreachableInst(());

impl UnreachableInst {
    pub(in crate::ir) fn new() -> Self {
        Self(())
    }
}

impl Terminator for UnreachableInst {
    fn targets(&self) -> &[BlockWithParams] {
        &[]
    }

    fn __operands(&self) -> &[Value] {
        &[]
    }
}

/// Models a return from a function
///
/// ```raw
/// ret i64 %0
/// ```
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct RetInst {
    value: Option<Value>,
}

impl RetInst {
    pub(in crate::ir) fn new(val: Option<Value>) -> Self {
        Self { value: val }
    }

    /// Gets the value being returned, if any.
    pub fn value(&self) -> Option<Value> {
        self.value
    }
}

impl Terminator for RetInst {
    fn targets(&self) -> &[BlockWithParams] {
        &[]
    }

    fn __operands(&self) -> &[Value] {
        match &self.value {
            Some(val) => slice::from_ref(val),
            None => &[],
        }
    }
}

/// Models a general arithmetic instruction
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct ArithmeticInst<const COMMUTATIVE: bool> {
    output: Type,
    operands: [Value; 2],
}

impl<const C: bool> ArithmeticInst<C> {
    pub(in crate::ir) fn new(output: Type, lhs: Value, rhs: Value) -> Self {
        Self {
            output,
            operands: [lhs, rhs],
        }
    }
}

impl<const C: bool> Instruction for ArithmeticInst<C> {
    fn operands(&self) -> &[Value] {
        &self.operands
    }

    fn result_ty(&self) -> Option<Type> {
        Some(self.output)
    }
}

impl<const C: bool> BinaryInst for ArithmeticInst<C> {
    fn is_commutative(&self) -> bool {
        C
    }
}

/// Models a general arithmetic instruction that is commutative (e.g. `iadd`, `imul`)
pub type CommutativeArithInst = ArithmeticInst<true>;

/// Models a general arithmetic instruction that isn't commutative (e.g. `isub`, `sdiv`)
pub type ArithInst = ArithmeticInst<false>;

/// Models a generalized cast instruction
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct CastInst {
    output: Type,
    operand: Value,
}

impl CastInst {
    pub(in crate::ir) fn new(output: Type, operand: Value) -> Self {
        Self { operand, output }
    }
}

impl Instruction for CastInst {
    fn operands(&self) -> &[Value] {
        slice::from_ref(&self.operand)
    }

    fn result_ty(&self) -> Option<Type> {
        Some(self.output)
    }
}

impl UnaryInst for CastInst {}

/// Models any unary floating-point arithmetic instructions (e.g. `fneg`)
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct FloatUnaryInst {
    output: Type,
    operand: Value,
}

impl FloatUnaryInst {
    pub(in crate::ir) fn new(output: Type, operand: Value) -> Self {
        Self { operand, output }
    }
}

impl Instruction for FloatUnaryInst {
    fn operands(&self) -> &[Value] {
        slice::from_ref(&self.operand)
    }

    fn result_ty(&self) -> Option<Type> {
        Some(self.output)
    }
}

impl UnaryInst for FloatUnaryInst {}

/// Models a direct function call to a known function
///
/// ```raw
/// %1 = call i32 @puts(ptr %0)
/// ```
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct CallInst {
    output: PackedOption<Type>,
    operands: TinyArray<Value, 2>,
}

assert_eq_size!(CallInst, [u64; 3]);

impl CallInst {
    pub(in crate::ir) fn new(output: Option<Type>, sig: Sig, callee: Func, args: &[Value]) -> Self {
        let sig = iter::once(Value::raw_from(sig));
        let callee = iter::once(Value::raw_from(callee));
        let args = args.iter().copied();

        Self {
            output: output.into(),
            operands: TinyArray::from_iter(sig.chain(callee).chain(args)),
        }
    }

    /// Gets the function signature
    pub fn sig(&self) -> Sig {
        self.operands[0].raw_into()
    }

    /// Gets the function being called
    pub fn callee(&self) -> Func {
        // we take the underlying data of the first key and convert it
        // into a function key instead, since that's what it actually is
        self.operands[1].raw_into()
    }

    /// Gets the arguments being passed into the function
    pub fn args(&self) -> &[Value] {
        match self.operands.get(2..) {
            Some(args) => args,
            None => &[],
        }
    }
}

impl Instruction for CallInst {
    fn operands(&self) -> &[Value] {
        self.args()
    }

    fn result_ty(&self) -> Option<Type> {
        self.output.expand()
    }
}

/// Models an indirect call to a function stored in a pointer.
///
/// ```raw
/// %2 = indirectcall void (i32), ptr %0(i32 %1)
/// ```
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct IndirectCallInst {
    output: PackedOption<Type>,
    operands: TinyArray<Value, 2>,
}

impl IndirectCallInst {
    pub(in crate::ir) fn new(
        output: Option<Type>,
        signature: Sig,
        callee: Value,
        args: &[Value],
    ) -> Self {
        let sig = iter::once(Value::raw_from(signature));
        let callee = iter::once(callee);
        let args = args.iter().copied();

        Self {
            output: output.into(),
            operands: TinyArray::from_iter(sig.chain(callee).chain(args)),
        }
    }

    /// Gets the function signature
    pub fn sig(&self) -> Sig {
        self.operands[0].raw_into()
    }

    /// Gets the function pointer being called
    pub fn callee(&self) -> Value {
        self.operands[1]
    }

    /// Gets the arguments being passed to the call
    pub fn args(&self) -> &[Value] {
        match self.operands.get(2..) {
            Some(args) => args,
            None => &[],
        }
    }
}

impl Instruction for IndirectCallInst {
    fn operands(&self) -> &[Value] {
        &self.operands
    }

    fn result_ty(&self) -> Option<Type> {
        self.output.expand()
    }
}

/// Models an `alloca` stack allocation
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct AllocaInst {
    ty: Type,
}

impl AllocaInst {
    pub(in crate::ir) fn new(ty: Type) -> Self {
        Self { ty }
    }

    /// Returns the type being allocated on the stack
    pub fn alloc_ty(&self) -> Type {
        self.ty
    }
}

impl Instruction for AllocaInst {
    fn operands(&self) -> &[Value] {
        &[]
    }

    fn result_ty(&self) -> Option<Type> {
        Some(Type::ptr())
    }
}

/// Models extracting a field from an aggregate
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct LoadInst {
    pointer: Value,
    output: Type,
}

impl LoadInst {
    pub(in crate::ir) fn new(pointer: Value, output: Type) -> Self {
        Self { pointer, output }
    }

    /// Gets the pointer being loaded
    pub fn pointer(&self) -> Value {
        self.pointer
    }
}

impl Instruction for LoadInst {
    fn operands(&self) -> &[Value] {
        slice::from_ref(&self.pointer)
    }

    fn result_ty(&self) -> Option<Type> {
        Some(self.output)
    }
}

/// Models a `store` instruction
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct StoreInst {
    operands: [Value; 2],
}

impl StoreInst {
    pub(in crate::ir) fn new(pointer: Value, val: Value) -> Self {
        Self {
            operands: [pointer, val],
        }
    }

    /// Gets the pointer being written to
    pub fn pointer(&self) -> Value {
        self.operands[0]
    }

    /// Gets the value being stored
    pub fn stored(&self) -> Value {
        self.operands[1]
    }
}

impl Instruction for StoreInst {
    fn operands(&self) -> &[Value] {
        &self.operands
    }

    fn result_ty(&self) -> Option<Type> {
        None
    }
}

/// Models an `offset` instruction
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct OffsetInst {
    operands: [Value; 2],
    ty: Type,
}

impl OffsetInst {
    pub(in crate::ir) fn new(base: Value, offset: Value, ty: Type) -> Self {
        Self {
            operands: [base, offset],
            ty,
        }
    }

    /// Gets the base of the new address
    pub fn base(&self) -> Value {
        self.operands[0]
    }

    /// Gets the offset being applied to [`Self::base`]
    pub fn offset(&self) -> Value {
        self.operands[1]
    }

    /// Gets the type being pointed to by [`Self::base`]. This affects how
    /// [`Self::offset`] is multiplied, similar to C pointer arithmetic
    pub fn offset_ty(&self) -> Type {
        self.ty
    }
}

impl Instruction for OffsetInst {
    fn operands(&self) -> &[Value] {
        &self.operands
    }

    fn result_ty(&self) -> Option<Type> {
        Some(Type::ptr())
    }
}

/// Models extracting a field from an aggregate
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct ExtractInst {
    operand: Value,
    output: Type,
    index: u64,
}

impl ExtractInst {
    pub(in crate::ir) fn new(output: Type, operand: Value, index: u64) -> Self {
        Self {
            operand,
            output,
            index,
        }
    }

    /// Gets the value being extracted from
    pub fn aggregate(&self) -> Value {
        self.operand
    }

    /// Gets the index of the field being extracted
    pub fn index(&self) -> u64 {
        self.index
    }
}

impl Instruction for ExtractInst {
    fn operands(&self) -> &[Value] {
        slice::from_ref(&self.operand)
    }

    fn result_ty(&self) -> Option<Type> {
        Some(self.output)
    }
}

/// Models setting a field in an aggregate
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct InsertInst {
    aggregate: Type,
    operands: [Value; 2],
    index: u64,
}

impl InsertInst {
    pub(in crate::ir) fn new(agg_ty: Type, aggregate: Value, value: Value, index: u64) -> Self {
        Self {
            aggregate: agg_ty,
            operands: [aggregate, value],
            index,
        }
    }

    /// Gets the aggregate being inserted to
    pub fn aggregate(&self) -> Value {
        self.operands[0]
    }

    /// Gets the value being inserted
    pub fn value(&self) -> Value {
        self.operands[1]
    }

    /// Gets the index of the field being inserted to
    pub fn index(&self) -> u64 {
        self.index
    }
}

impl Instruction for InsertInst {
    fn operands(&self) -> &[Value] {
        &self.operands
    }

    fn result_ty(&self) -> Option<Type> {
        Some(self.aggregate)
    }
}

/// Models getting a pointer to the field of an aggregate
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct ElemPtrInst {
    aggregate: Type,
    base: Value,
    index: u64,
}

impl ElemPtrInst {
    pub(in crate::ir) fn new(aggregate: Type, base: Value, index: u64) -> Self {
        Self {
            aggregate,
            base,
            index,
        }
    }

    /// Gets the type being pointed to by [`Self::base`]
    pub fn aggregate_ty(&self) -> Type {
        self.aggregate
    }

    /// Returns the pointer to the aggregate being indexed into
    pub fn base(&self) -> Value {
        self.base
    }

    /// Gets the index of the field of [`Self::base`] the resulting pointer will point at
    pub fn index(&self) -> u64 {
        self.index
    }
}

impl Instruction for ElemPtrInst {
    fn operands(&self) -> &[Value] {
        slice::from_ref(&self.base)
    }

    fn result_ty(&self) -> Option<Type> {
        Some(Type::ptr())
    }
}

/// Models an `iconst` instruction.
///
/// ```raw
/// %0 = iconst i32 13
/// ```
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct IConstInst {
    constant: u64,
    mask: u64,
    ty: Type,
}

impl IConstInst {
    pub(in crate::ir) fn new(ty: Type, constant: u64) -> Self {
        Self {
            constant,
            ty,
            mask: ty.unwrap_int().mask(),
        }
    }

    /// Gets the actual const value being yielded as an unsigned integer
    pub fn value(&self) -> u64 {
        self.constant & self.mask
    }
}

impl Instruction for IConstInst {
    fn operands(&self) -> &[Value] {
        &[]
    }

    fn result_ty(&self) -> Option<Type> {
        Some(self.ty)
    }
}

/// Models an `fconst` instruction.
///
/// ```raw
/// %0 = fconst f32 0.0
/// ```
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct FConstInst {
    constant: u64,
    mask: u64,
    ty: Type,
}

impl FConstInst {
    pub(in crate::ir) fn new(ty: Type, constant: u64) -> Self {
        Self {
            constant,
            ty,
            mask: match ty.unwrap_float().format() {
                FloatFormat::Double => Type::i64().unwrap_int().mask(),
                FloatFormat::Single => Type::i32().unwrap_int().mask(),
            },
        }
    }

    /// Gets the byte value of the floating-point constant
    pub fn value(&self) -> u64 {
        self.constant & self.mask
    }
}

impl Instruction for FConstInst {
    fn operands(&self) -> &[Value] {
        &[]
    }

    fn result_ty(&self) -> Option<Type> {
        Some(self.ty)
    }
}

/// Models a `bconst` instruction.
///
/// ```raw
/// %0 = bconst bool true
/// ```
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct BConstInst {
    constant: bool,
}

impl BConstInst {
    pub(in crate::ir) fn new(constant: bool) -> Self {
        Self { constant }
    }

    /// Gets the actual constant value
    pub fn value(&self) -> bool {
        self.constant
    }
}

impl Instruction for BConstInst {
    fn operands(&self) -> &[Value] {
        &[]
    }

    fn result_ty(&self) -> Option<Type> {
        Some(Type::bool())
    }
}

/// Models an `undef` instruction.
///
/// ```raw
/// %0 = undef i32
/// ```
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct UndefConstInst {
    ty: Type,
}

impl UndefConstInst {
    pub(in crate::ir) fn new(ty: Type) -> Self {
        Self { ty }
    }
}

impl Instruction for UndefConstInst {
    fn operands(&self) -> &[Value] {
        &[]
    }

    fn result_ty(&self) -> Option<Type> {
        Some(self.ty)
    }
}

/// Models a `null` instruction.
///
/// ```raw
/// %0 = null i32
/// ```
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct NullConstInst {
    ty: Type,
}

impl NullConstInst {
    pub(in crate::ir) fn new(ty: Type) -> Self {
        Self { ty }
    }
}

impl Instruction for NullConstInst {
    fn operands(&self) -> &[Value] {
        &[]
    }

    fn result_ty(&self) -> Option<Type> {
        Some(self.ty)
    }
}

/// Models aa `globaladdr` instruction.
///
/// ```raw
/// %0 = globaladdr @printf
/// ```
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct GlobalAddrInst {
    name: String,
}

impl GlobalAddrInst {
    pub(in crate::ir) fn new(name: String) -> Self {
        Self { name }
    }

    /// Gets the name of the symbol being referenced
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Instruction for GlobalAddrInst {
    fn operands(&self) -> &[Value] {
        &[]
    }

    fn result_ty(&self) -> Option<Type> {
        Some(Type::ptr())
    }
}
