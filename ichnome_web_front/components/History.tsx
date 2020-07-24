import { IchHistory } from "@/api/types";
import NamespaceLink from "./NamespaceLink";
import StatLink from "./StatLink";
import ObjectLink from "./ObjectLink";

type Props = {
  history: IchHistory;
};

export const History: React.FC<Props> = ({
  history: { namespace_id, path, version, status, mtime, digest, created_at, updated_at },
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
      <li>History Created At: {created_at}</li>
      <li>History Updated At: {updated_at}</li>
    </ul>
  );
};

export default History;
