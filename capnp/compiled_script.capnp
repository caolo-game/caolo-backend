@0xbb79483d16c260a9;

using CompiledProgram = import "compiled_program.capnp".CompiledProgram;

struct CompiledScript
{
    scriptId @0: Data;
    compiled @1: CompiledProgram;
}

