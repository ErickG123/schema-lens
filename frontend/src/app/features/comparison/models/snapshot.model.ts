export interface SnapshotColumn {
  name: string;
  ordinal_position: number;
  data_type: string;
  max_length: number;
  precision: number;
  scale: number;
  is_nullable: boolean;
  is_identity: boolean;
  identity_seed?: number | null;
  identity_increment?: number | null;
  is_computed: boolean;
  computed_definition?: string | null;
  collation?: string | null;
}

export interface SnapshotForeignKey {
  name: string;
  referenced_schema: string;
  referenced_table: string;
  column_mappings: [string, string][];
  delete_rule: string;
  update_rule: string;
  is_disabled: boolean;
}

export interface SnapshotTable {
  id: { schema: string; name: string };
  columns: SnapshotColumn[];
  primary_key?: { name: string; is_clustered: boolean; columns: string[] } | null;
  foreign_keys: SnapshotForeignKey[];
  check_constraints: { name: string; definition: string; is_disabled: boolean }[];
  default_constraints: { name: string; column_name: string; definition: string }[];
  unique_constraints: { name: string; is_clustered: boolean; columns: string[] }[];
  indexes: { name: string; is_unique: boolean; is_clustered: boolean; key_columns: string[]; key_columns_descending: boolean[]; included_columns: string[]; filter_definition?: string | null }[];
}
