using System;
using System.Linq;
using Mono.Cecil;
using Mono.Cecil.Cil;
class PatchNativeDialogue {
 static void Main(string[] a) {
  var resolver=new DefaultAssemblyResolver();resolver.AddSearchDirectory(a[3]);
  var game=AssemblyDefinition.ReadAssembly(a[0],new ReaderParameters{AssemblyResolver=resolver});
  var helper=AssemblyDefinition.ReadAssembly(a[1]);var mod=game.MainModule;
  var methods=helper.MainModule.GetType("NativeDialogue").Methods;
  Func<string,MethodReference> import=n=>mod.ImportReference(methods.Single(x=>x.Name==n));
  foreach(var pair in new[]{new[]{"JLC.JLCSubtitleControls","SetSubtitle"},new[]{"JLC.JLCResponseButton","SetUp"},new[]{"JLC.JLCSubtitleTypeOn","ToggleMiniSubtitles"}}) {
   var m=mod.GetType(pair[0]).Methods.Single(x=>x.Name==pair[1]);var il=m.Body.GetILProcessor();int flags=0;
   foreach(var i in m.Body.Instructions.ToArray()) {
    var f=i.Operand as FieldReference;var r=i.Operand as MethodReference;
    if(f!=null && f.FullName=="System.Boolean JLC.FontSet::isRightToLeft") { i.OpCode=OpCodes.Call;i.Operand=import("IsRtl");flags++; }
    if(r!=null && r.DeclaringType.FullName=="RTLTMPro.RTLSupport" && r.Name=="FixRTL") i.Operand=import("FixRTL");
    if(r!=null && r.Name=="ToString" && i.Previous.Operand is FieldReference && ((FieldReference)i.Previous.Operand).Name=="finalText")
     il.InsertAfter(i,il.Create(OpCodes.Call,import("Encode")));
   }
   if(flags!=1)throw new Exception("Expected one RTL flag in "+m.FullName);
  }
  foreach(var pair in new[]{new[]{"JLC.JLCSubtitleTypeOn","textMeshPro"},new[]{"JLC.JLCResponseButton","label"}}) {
   var type=mod.GetType(pair[0]);var m=type.Methods.Single(x=>x.Name=="set_Text");var il=m.Body.GetILProcessor();var first=m.Body.Instructions[0];
   il.InsertBefore(first,il.Create(OpCodes.Ldarg_0));il.InsertBefore(first,il.Create(OpCodes.Ldfld,type.Fields.Single(x=>x.Name==pair[1])));il.InsertBefore(first,il.Create(OpCodes.Call,import("Configure")));
  }
  foreach(var type in mod.Types) foreach(var method in type.Methods.Where(x=>x.HasBody))
   foreach(var i in method.Body.Instructions) if(i.OpCode.OperandType==OperandType.ShortInlineBrTarget) {
    var name=i.OpCode.Name.Substring(0,i.OpCode.Name.Length-2);
    i.OpCode=(OpCode)typeof(OpCodes).GetFields().Select(f=>f.GetValue(null)).First(v=>v is OpCode && ((OpCode)v).Name==name);
   }
  game.Write(a[2]);
 }
}
