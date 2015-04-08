/*
   american fuzzy lop - LLVM instrumentation pass
   ----------------------------------------------

   Written by Laszlo Szekeres <lszekeres@google.com>,
              Keegan McAllister <mcallister.keegan@gmail.com>

   Copyright 2015 Google Inc. All rights reserved.

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at:

     http://www.apache.org/licenses/LICENSE-2.0

*/

#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/LegacyPassManager.h"
#include "llvm/IR/Module.h"
#include "llvm/Support/Debug.h"
#include "llvm/Transforms/IPO/PassManagerBuilder.h"

using namespace llvm;

namespace {
class AFLCoverage : public ModulePass {
 public:
  static char ID;
  AFLCoverage() : ModulePass(ID) {}

  bool runOnModule(Module &M) override;
};
}

bool AFLCoverage::runOnModule(Module &M) {
  LLVMContext &C = M.getContext();

  IntegerType *Int8Ty = IntegerType::getInt8Ty(C);
  IntegerType *Int16Ty = IntegerType::getInt16Ty(C);
  IntegerType *Int64Ty = IntegerType::getInt64Ty(C);

  const unsigned int mapsize = 1 << 16;

  // Get uniform distrubution of 16 bit numbers
  std::random_device rand_dev;
  std::mt19937 generator(rand_dev());
  std::uniform_int_distribution<int> distribution(0, mapsize - 1);

  // Globals for map and prev_loc
  GlobalVariable *AFLMapPtr =
      new GlobalVariable(M, PointerType::get(Int8Ty, 0), false,
                         GlobalValue::ExternalLinkage, 0, "__afl_area_ptr");

  GlobalVariable *AFLPrevLoc = new GlobalVariable(
      M, Int16Ty, false, GlobalValue::ExternalLinkage, 0, "__afl_prev_loc");

  // Instrument all blocks
  for (auto &F : M)
    for (auto &BB : F) {
      BasicBlock::iterator IP = BB.getFirstInsertionPt();
      IRBuilder<> IRB(IP);

      // Get randmon number and cur_loc
      unsigned int random = distribution(generator);
      ConstantInt *CurLoc = ConstantInt::get(Int64Ty, random);

      // Load prev_loc
      LoadInst *PrevLoc = IRB.CreateLoad(AFLPrevLoc);
      PrevLoc->setMetadata(M.getMDKindID("nosanitize"), MDNode::get(C, None));
      Value *PrevLocCasted = IRB.CreateZExt(PrevLoc, IRB.getInt64Ty());

      // Load map pointer
      LoadInst *MapPtr = IRB.CreateLoad(AFLMapPtr);
      MapPtr->setMetadata(M.getMDKindID("nosanitize"), MDNode::get(C, None));
      Value *MapPtrIdx =
          IRB.CreateGEP(MapPtr, IRB.CreateXor(PrevLocCasted, CurLoc));

      // Update counter
      LoadInst *Counter = IRB.CreateLoad(MapPtrIdx);
      Counter->setMetadata(M.getMDKindID("nosanitize"), MDNode::get(C, None));
      Value *Incr = IRB.CreateAdd(Counter, ConstantInt::get(Int8Ty, 1));
      IRB.CreateStore(Incr, MapPtrIdx)
          ->setMetadata(M.getMDKindID("nosanitize"), MDNode::get(C, None));

      // Update prev_loc
      StoreInst *Store =
          IRB.CreateStore(ConstantInt::get(Int16Ty, random >> 1), AFLPrevLoc);
      Store->setMetadata(M.getMDKindID("nosanitize"), MDNode::get(C, None));
    }

  return true;
}

char AFLCoverage::ID = 0;

static RegisterPass<AFLCoverage> RegisterAFLPass("afl-coverage",
    "American Fuzzy Lop Instrumentation");
