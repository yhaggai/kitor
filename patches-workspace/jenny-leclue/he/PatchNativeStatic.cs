using System;using System.Linq;using Mono.Cecil;using Mono.Cecil.Cil;
class PatchNativeStatic {
 static void Main(string[]a){
 var resolver=new DefaultAssemblyResolver();resolver.AddSearchDirectory(a[3]);
 var g=AssemblyDefinition.ReadAssembly(a[0],new ReaderParameters{AssemblyResolver=resolver});var h=AssemblyDefinition.ReadAssembly(a[1]);var mod=g.MainModule;
 Func<string,MethodReference> helper=n=>mod.ImportReference(h.MainModule.GetType("NativeStatic").Methods.Single(m=>m.Name==n));
 var lookup=mod.GetType("JLC.LocalisationManager").Methods.Single(m=>m.Name=="GetLocalisedValue");
 if(lookup.Body.Instructions.Any(i=>i.Operand is MethodReference && ((MethodReference)i.Operand).DeclaringType.Name=="NativeStatic"))throw new Exception("Static RTL already patched");
 // Mutate each ret into ldarg.1 so branches to ret also execute the hook.
 foreach(var i in lookup.Body.Instructions.Where(i=>i.OpCode==OpCodes.Ret).ToArray()){
 var il=lookup.Body.GetILProcessor();i.OpCode=OpCodes.Ldarg_1;var call=il.Create(OpCodes.Call,helper("Process"));il.InsertAfter(i,call);il.InsertAfter(call,il.Create(OpCodes.Ret));}
 var refresh=mod.GetType("JLC.LocalisedText").Methods.Single(m=>m.Name=="OnChangeLanguage");
 foreach(var i in refresh.Body.Instructions.Where(i=>i.OpCode==OpCodes.Ret).ToArray()){
 var il=refresh.Body.GetILProcessor();i.OpCode=OpCodes.Ldarg_0;var call=il.Create(OpCodes.Call,helper("Refresh"));il.InsertAfter(i,call);il.InsertAfter(call,il.Create(OpCodes.Ret));}
 var speed=mod.GetType("JLC.ClassroomTypewriter").Methods.Single(m=>m.Name=="GetCharsPerSecond");int count=0;
 foreach(var i in speed.Body.Instructions.ToArray())if(i.Operand is FieldReference && ((FieldReference)i.Operand).FullName=="System.String JLC.LanguageInfo::languageCode") {speed.Body.GetILProcessor().InsertAfter(i,speed.Body.GetILProcessor().Create(OpCodes.Call,helper("TypewriterLanguage")));count++;}
 if(count!=1)throw new Exception("Typewriter language lookup mismatch");
 foreach(var m in new[]{lookup,refresh,speed})foreach(var i in m.Body.Instructions)if(i.OpCode.OperandType==OperandType.ShortInlineBrTarget){var name=i.OpCode.Name.Substring(0,i.OpCode.Name.Length-2);i.OpCode=(OpCode)typeof(OpCodes).GetFields().Select(f=>f.GetValue(null)).First(v=>v is OpCode && ((OpCode)v).Name==name);}
 int setters=0;
 foreach(var type in mod.Types.Where(t=>t.Namespace=="JLC" && t.Name.StartsWith("MenuButton")))
 foreach(var method in type.Methods.Where(m=>m.HasBody))
 foreach(var i in method.Body.Instructions) {
 var call=i.Operand as MethodReference;
 if(call!=null && call.DeclaringType.FullName=="TMPro.TMP_Text" && call.Name=="SetText" && call.Parameters.Count==1 && call.Parameters[0].ParameterType.FullName=="System.String") {
 i.OpCode=OpCodes.Call;i.Operand=helper("SetMenuText");setters++;Console.WriteLine("Dynamic menu RTL: "+method.FullName);
 }
 }
 if(setters<5)throw new Exception("Missing dynamic menu setters");
 mod.Write(a[2]);
 }
}
