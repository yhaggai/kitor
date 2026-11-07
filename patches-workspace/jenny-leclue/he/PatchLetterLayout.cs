using System.Linq;
using Mono.Cecil;
using Mono.Cecil.Cil;

// Apply to an existing Kitor-patched assembly, preserving all other hooks.
class PatchLetterLayout {
    static void Main(string[] args) {
        var resolver = new DefaultAssemblyResolver();
        resolver.AddSearchDirectory(args[3]);
        var game = AssemblyDefinition.ReadAssembly(args[0], new ReaderParameters { AssemblyResolver = resolver });
        var helper = AssemblyDefinition.ReadAssembly(args[1]);
        var method = game.MainModule.GetType("JLC.LocalisedLayout").Methods.Single(x => x.Name == "ChangeLanguage");
        if (method.Body.Instructions.Any(x => x.Operand is MethodReference &&
            ((MethodReference)x.Operand).Name == "LetterLayoutLanguage"))
            throw new System.InvalidOperationException("Letter layout hook already installed");
        var store = method.Body.Instructions.First(x => x.OpCode == OpCodes.Stloc_0);
        var il = method.Body.GetILProcessor();
        il.InsertBefore(store, il.Create(OpCodes.Ldarg_0));
        il.InsertBefore(store, il.Create(OpCodes.Call, game.MainModule.ImportReference(
            helper.MainModule.GetType("HebrewShortcut").Methods.Single(x => x.Name == "LetterLayoutLanguage"))));
        game.Write(args[2]);
    }
}
