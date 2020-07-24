import Link from "next/link";
import { uri } from "../utils/uri";
import { IchNamespace } from "@/api/types";
import NamespaceLink from "./NamespaceLink";
import ObjectLink from "./ObjectLink";

type Props = {
  namespace: IchNamespace;
};

export const Namespace: React.FC<Props> = ({ namespace: { id, type, url, digest, created_at, updated_at } }) => {
  return (
    <ul>
      <li>
        ID: <NamespaceLink namespaceId={id} /> (Stats: <NamespaceLink namespaceId={id} family={"stats"} />)
      </li>
      <li>Type: {type}</li>
      <li>URL: {url}</li>
      {digest != null ? (
        <li>
          Digest: <ObjectLink digest={digest} />
        </li>
      ) : undefined}
      <li>Namespace Created At: {created_at}</li>
      <li>Namespace Updated At: {updated_at}</li>
    </ul>
  );
};

export default Namespace;
