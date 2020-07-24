export type IchNamespace = {
  id: string;
  url: string;
  type: number;
  history_id?: number;
  version?: number;
  status?: number;
  mtime?: string;
  object_id?: number;
  digest?: string;
  size?: number;
  created_at: string;
  updated_at: string;
};

export type IchStat = {
  id: number;
  namespace_id: string;
  path: string;
  history_id: number;
  version: number;
  status: number;
  mtime?: string;
  object_id?: number;
  digest?: string;
  size?: number;
  created_at: string;
  updated_at: string;
};

export type IchHistory = {
  id: number;
  namespace_id: string;
  path: string;
  version: number;
  status: number;
  mtime?: string;
  object_id?: number;
  digest?: string;
  created_at: string;
  updated_at: string;
};

export type IchObject = {
  id: number;
  digest: string;
  size: number;
  git_object_id: string;
};

export type GetStatsResponse = {
  namespace: IchNamespace;
  stats: IchStat[];
};

export type GetStatResponse = {
  namespace: IchNamespace;
  stat: IchStat;
  histories?: IchHistory[];
  objects?: { [k: string]: IchObject };
  eq_stats?: IchStat[];
};

export type GetObjectResponse = {
  object: IchObject;
  namespace_id?: string;
  stats?: IchStat[];
  histories?: IchHistory[];
};

export type GetNamespacesResponse = {
  namespaces: IchNamespace[];
};

export type GetNamespaceResponse = {
  namespace: IchNamespace;
  stat?: IchStat;
  histories?: IchHistory[];
  objects?: { [k: string]: IchObject };
};
