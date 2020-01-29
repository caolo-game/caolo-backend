@0xc2cd5bd1231431ea;

using CompiledLabel = import "compiled_label.capnp".CompiledLabel;

struct CompiledProgram
{
    bytecode     @0: Data;
    labelsKeys   @1 : List(Int32);
    labelsValues @2 : List(CompiledLabel);
}
