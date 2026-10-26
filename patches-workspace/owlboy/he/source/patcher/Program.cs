using Mono.Cecil;
using Mono.Cecil.Cil;
if(args.Length!=3)throw new ArgumentException("Usage: Patcher ORIGINAL_EXE HEBREW_RUNTIME_DLL OUTPUT_EXE");
var resolver=new DefaultAssemblyResolver();resolver.AddSearchDirectory(Path.GetDirectoryName(Path.GetFullPath(args[0]))!);
using var game=AssemblyDefinition.ReadAssembly(args[0],new ReaderParameters{AssemblyResolver=resolver});
// Read into memory so the helper can be rewritten in place after its direct
// bridge to the newly added game method has been bound.
using var helper=AssemblyDefinition.ReadAssembly(args[1],new ReaderParameters{AssemblyResolver=resolver,InMemory=true});
var module=game.MainModule;var runtime=helper.MainModule.GetType("OwlboyHebrew.Runtime");
MethodReference Import(string name)=>module.ImportReference(runtime.Methods.Single(m=>m.Name==name));
var font=module.GetType("Owlboy.Font");
if(font.Methods.Any(m=>m.Name=="HebrewOriginalDraw"))throw new InvalidOperationException("Already patched: restore original first");
var draw=font.Methods.Single(m=>m.Name=="InternalDraw");
if(draw.Parameters.Count!=11)throw new InvalidOperationException("Unsupported Font.InternalDraw signature");
var original=new MethodDefinition("HebrewOriginalDraw",MethodAttributes.Public|MethodAttributes.HideBySig,module.TypeSystem.Void);
foreach(var p in draw.Parameters)original.Parameters.Add(new ParameterDefinition(p.Name,p.Attributes,p.ParameterType));
font.Methods.Add(original);
// A MethodBody retains its owning method. Clone it instead of transferring the
// body object, and remap every body-owned operand and exception boundary.
var sourceBody=draw.Body;
var clonedBody=new MethodBody(original){InitLocals=sourceBody.InitLocals,MaxStackSize=sourceBody.MaxStackSize};
original.Body=clonedBody;
var variables=new Dictionary<VariableDefinition,VariableDefinition>();
foreach(var variable in sourceBody.Variables){var copy=new VariableDefinition(variable.VariableType);variables.Add(variable,copy);clonedBody.Variables.Add(copy);}
var instructions=new Dictionary<Instruction,Instruction>();
foreach(var instruction in sourceBody.Instructions){var copy=Instruction.Create(OpCodes.Nop);copy.OpCode=instruction.OpCode;instructions.Add(instruction,copy);clonedBody.Instructions.Add(copy);}
foreach(var instruction in sourceBody.Instructions){
 instructions[instruction].Operand=instruction.Operand switch {
  Instruction target=>instructions[target],
  Instruction[] targets=>targets.Select(target=>instructions[target]).ToArray(),
  VariableDefinition variable=>variables[variable],
  ParameterDefinition parameter=>parameter.Index<0?clonedBody.ThisParameter:original.Parameters[parameter.Index],
  _=>instruction.Operand
 };
}
Instruction? Boundary(Instruction? instruction)=>instruction==null?null:instructions[instruction];
foreach(var handler in sourceBody.ExceptionHandlers)clonedBody.ExceptionHandlers.Add(new ExceptionHandler(handler.HandlerType){
 CatchType=handler.CatchType,TryStart=Boundary(handler.TryStart),TryEnd=Boundary(handler.TryEnd),
 HandlerStart=Boundary(handler.HandlerStart),HandlerEnd=Boundary(handler.HandlerEnd),FilterStart=Boundary(handler.FilterStart)
});
draw.Body=new MethodBody(draw);var il=draw.Body.GetILProcessor();
void Args(){il.Emit(OpCodes.Ldarg_0);foreach(var p in draw.Parameters)il.Emit(OpCodes.Ldarg,p);}
Args();il.Emit(OpCodes.Call,Import("TryDraw"));var fallback=Instruction.Create(OpCodes.Nop);il.Emit(OpCodes.Brfalse,fallback);il.Emit(OpCodes.Ret);il.Append(fallback);Args();il.Emit(OpCodes.Call,original);il.Emit(OpCodes.Ret);
var renderer=module.GetType("Owlboy.ScreenManagement.Renderers.SpeechBubbleRenderer");int slices=0;
foreach(string name in new[]{"Draw","CreateRenderTargets"}){
 var method=renderer.Methods.Single(m=>m.Name==name);var body=method.Body;var edit=body.GetILProcessor();
 foreach(var ins in body.Instructions.ToArray()){
  if(ins.OpCode==OpCodes.Callvirt && ins.Operand is MethodReference target && target.DeclaringType.FullName=="System.String" && target.Name=="Substring" && target.Parameters.Count==2){ins.OpCode=OpCodes.Call;ins.Operand=Import("Slice");slices++;}
  if(ins.OpCode==OpCodes.Ret){ins.OpCode=OpCodes.Call;ins.Operand=Import("End");edit.InsertAfter(ins,Instruction.Create(OpCodes.Ret));}
 }
 var first=body.Instructions[0];edit.InsertBefore(first,Instruction.Create(OpCodes.Ldarg_0));edit.InsertBefore(first,Instruction.Create(OpCodes.Ldarg,method.Parameters[1]));edit.InsertBefore(first,Instruction.Create(OpCodes.Call,Import("Begin")));
}
if(slices!=2)throw new InvalidOperationException($"Unsupported speech renderer: expected 2 substring sites, found {slices}");
// Insertion can make a short jump exceed its signed-byte reach.
var longOps=typeof(OpCodes).GetFields(System.Reflection.BindingFlags.Public|System.Reflection.BindingFlags.Static).Where(f=>f.FieldType==typeof(OpCode)).Select(f=>(OpCode)f.GetValue(null)!).ToDictionary(o=>o.Name);
foreach(var method in renderer.Methods.Where(m=>m.HasBody))foreach(var instruction in method.Body.Instructions)
 if(instruction.Operand is Instruction && instruction.OpCode.Name.EndsWith(".s"))instruction.OpCode=longOps[instruction.OpCode.Name[..^2]];
// The bundled Mono can crash while constructing reflection/delegate wrappers
// with FNA value-type arguments. Bind an ordinary managed call instead.
var bridge=runtime.Methods.Single(m=>m.Name=="OriginalDraw");
if(!bridge.IsStatic || bridge.Parameters.Count!=draw.Parameters.Count+1 || bridge.ReturnType.FullName!="System.Void")
 throw new InvalidOperationException("Unsupported runtime OriginalDraw bridge signature");
if(bridge.Parameters[0].ParameterType.FullName!=font.FullName ||
 bridge.Parameters.Skip(1).Where((p,i)=>p.ParameterType.FullName!=draw.Parameters[i].ParameterType.FullName).Any())
 throw new InvalidOperationException("Runtime OriginalDraw bridge argument types differ from the original method");
bridge.Body=new MethodBody(bridge);
var bridgeIl=bridge.Body.GetILProcessor();
foreach(var parameter in bridge.Parameters)bridgeIl.Emit(OpCodes.Ldarg,parameter);
bridgeIl.Emit(OpCodes.Call,helper.MainModule.ImportReference(original));
bridgeIl.Emit(OpCodes.Ret);
helper.Write(args[1]);
game.Write(args[2]);Console.WriteLine("Patched font drawing and speech contexts; original timing/logic preserved.");
