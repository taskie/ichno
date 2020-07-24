import { IchStat } from "@/api/types";
import NamespaceLink from "./NamespaceLink";
import StatLink from "./StatLink";
import ObjectLink from "./ObjectLink";

type Props = {
  stat: IchStat;
};

export const StatGroupItem: React.FC<Props> = ({
  stat: { namespace_id, path, version, mtime, size, digest, updated_at },
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
      {" / "}
      {size != null ? size : "Nothing"}
    </li>
  );
};

export default StatGroupItem;
