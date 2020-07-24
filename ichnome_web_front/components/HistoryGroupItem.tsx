import { IchHistory } from "@/api/types";
import NamespaceLink from "./NamespaceLink";
import StatLink from "./StatLink";
import ObjectLink from "./ObjectLink";

type Props = {
  history: IchHistory;
};

export const HistoryGroupItem: React.FC<Props> = ({
  history: { namespace_id, path, version, mtime, digest, updated_at },
}) => {
  return (
    <li>
      {digest != null ? <ObjectLink digest={digest} length={8} /> : undefined}
      {" / "}
      {mtime != null ? mtime : "Nothing"}
      {" / "}
      {updated_at}
      {" / "}
      <NamespaceLink namespaceId={namespace_id} />
      {" / "}
      <StatLink namespaceId={namespace_id} path={path} />
      {" / "}
      {version}
    </li>
  );
};

export default HistoryGroupItem;
