export type IchGroup = {
  id: number;
  name: string;
  url: string;
  type: number;
  history_id?: number;
  version?: number;
  status?: number;
  mtime?: string;
  footprint_id?: number;
  digest?: string;
  size?: number;
  created_at: string;
  updated_at: string;
};

export type IchStat = {
  id: number;
  group_name: string;
  path: string;
  history_id: number;
  version: number;
  status: number;
  mtime?: string;
  footprint_id?: number;
  digest?: string;
  size?: number;
  created_at: string;
  updated_at: string;
};

export type IchHistory = {
  id: number;
  group_name: string;
  path: string;
  version: number;
  status: number;
  mtime?: string;
  footprint_id?: number;
  digest?: string;
  created_at: string;
  updated_at: string;
};

export type IchFootprint = {
  id: number;
  digest: string;
  size: number;
  git_object_id: string;
};

export type GetStatsResponse = {
  group: IchGroup;
  stats: IchStat[];
  stats_count: number;
};

export type GetStatResponse = {
  group: IchGroup;
  stat: IchStat;
  histories?: IchHistory[];
  footprints?: { [k: string]: IchFootprint };
  eq_stats?: IchStat[];
};

export type GetFootprintResponse = {
  footprint: IchFootprint;
  group_id?: string;
  stats?: IchStat[];
  histories?: IchHistory[];
};

export type GetGroupsResponse = {
  groups: IchGroup[];
};

export type GetGroupResponse = {
  group: IchGroup;
  stat?: IchStat;
  histories?: IchHistory[];
  footprints?: { [k: string]: IchFootprint };
};

export type GetDiffResponse = {
  group1: IchGroup;
  group2: IchGroup;
  diff: { [k: string]: [number[], number[]] };
  stats: { [k: string]: IchStat };
  footprints: { [k: string]: IchFootprint };
};
