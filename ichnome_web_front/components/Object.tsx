import { IchObject } from "@/api/types";
import ObjectLink from "./ObjectLink";
import Digest from "./Digest";

type Props = {
  object: IchObject;
};

export const Object: React.FC<Props> = ({ object: { digest, size, git_object_id } }) => {
  return (
    <ul>
      <li>
        Digest: <ObjectLink digest={digest} />
      </li>
      <li>Size: {size}</li>
      <li>
        Git Object ID: <Digest digest={git_object_id} />
      </li>
    </ul>
  );
};

export default Object;
