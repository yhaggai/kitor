type Consts = {
  version: string;
};

let cachedConsts: Consts | null = null;

export async function getConsts(): Promise<Consts> {
  if (cachedConsts) {
    return cachedConsts;
  }

  const response = await fetch("/consts.json");
  if (!response.ok) {
    throw new Error("Failed to load consts.json");
  }

  const data = await response.json();
  cachedConsts = data;
  return data;
}

export async function getScummVMVersion(): Promise<string> {
  const consts = await getConsts();
  return consts.version;
}
