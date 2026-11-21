using Mono.Cecil;
using Mono.Cecil.Cil;
using System.IO;

if (args.Length != 2) throw new ArgumentException("Usage: PatchSaveRegex <input.dll> <output.dll>");
var resolver = new DefaultAssemblyResolver();
resolver.AddSearchDirectory(Path.GetDirectoryName(args[0])!);
using var assembly = AssemblyDefinition.ReadAssembly(args[0], new ReaderParameters { ReadSymbols = false, AssemblyResolver = resolver });
var matches = assembly.MainModule.Types.SelectMany(Flatten).SelectMany(t => t.Methods).Where(m => m.HasBody).SelectMany(m => m.Body.Instructions).Where(i => i.OpCode == OpCodes.Ldstr && i.Operand is string value && value.Contains("chapter") && value.Contains("version") && value.EndsWith(".sav")).ToArray();
Console.WriteLine($"save validator string matches={matches.Length}");
if (matches.Length != 2) throw new InvalidOperationException($"Expected two save pattern literals (manager cleanup and slot UI), got {matches.Length}");
var slash = (char)92;
var chapterClass = "[" + slash + "w" + slash + "p{Co}]+";
foreach (var instruction in matches)
{
    var value = (string)instruction.Operand;
    var chapterStart = value.IndexOf("(?'chapter'", StringComparison.Ordinal) + "(?'chapter'".Length;
    var chapterEnd = value.IndexOf(')', chapterStart);
    if (chapterStart < 0 || chapterEnd < chapterStart || !value.AsSpan(chapterEnd).StartsWith("){0,1}")) throw new InvalidOperationException("Unexpected save regex chapter group");
    var changed = value[..chapterStart] + chapterClass + value[chapterEnd..];
    var extension = changed.LastIndexOf(".sav", StringComparison.Ordinal);
    if (extension < 0) throw new InvalidOperationException("Save regex extension missing");
    changed = changed[..extension] + slash + changed[extension..] + "$";
    instruction.Operand = changed;
    Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(changed));
}
assembly.Write(args[1], new WriterParameters { WriteSymbols = false });
Console.WriteLine("Patched both save enumeration and slot-display regexes for PUA chapter names; filename suffix is anchored.");

static IEnumerable<TypeDefinition> Flatten(TypeDefinition type)
{
    yield return type;
    foreach (var nested in type.NestedTypes)
        foreach (var item in Flatten(nested)) yield return item;
}
