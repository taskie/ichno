import Link from "next/link";
import { uri } from "../utils/uri";

type Props = {
  namespaceId: string;
  path: string;
};

export const StatLink: React.FC<Props> = ({ namespaceId, path }) => (
  <Link href={uri`/stats/${namespaceId}/` + path}>
    <a>{path}</a>
  </Link>
);

export default StatLink;
