import { IchStat } from "@/api/types";
import NamespaceLink from "./NamespaceLink";
import StatLink from "./StatLink";
import ObjectLink from "./ObjectLink";

type Props = {
  stat: IchStat;
};

export const Stat: React.FC<Props> = ({
  stat: { namespace_id, path, version, status, mtime, digest, size, created_at, updated_at },
}) => {
  return (
    <ul>
      <li>
        Namespace: <NamespaceLink namespaceId={namespace_id} />
      </li>
      <li>
        Path: <StatLink namespaceId={namespace_id} path={path} />
      </li>
      <li>Version: {version}</li>
      <li>Status: {status}</li>
      {mtime != null ? <li>File Modified At: {mtime}</li> : undefined}
      {digest != null ? (
        <li>
          Digest: <ObjectLink digest={digest} />
        </li>
      ) : undefined}
      {size != null ? <li>Size: {size}</li> : undefined}
      <li>Stat Created At: {created_at}</li>
      <li>Stat Updated At: {updated_at}</li>
    </ul>
  );
};

export default Stat;
