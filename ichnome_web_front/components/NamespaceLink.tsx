import Link from "next/link";
import { uri } from "../utils/uri";

type Props = {
  namespaceId: string;
  family?: string;
};

export const NamespaceLink: React.FC<Props> = ({ namespaceId, family }) => {
  const href = family === "stats" ? uri`/stats/${namespaceId}` : uri`/namespaces/${namespaceId}`;
  return (
    <Link href={href}>
      <a>{namespaceId}</a>
    </Link>
  );
};

export default NamespaceLink;
