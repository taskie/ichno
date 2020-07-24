import { IchHistory } from "@/api/types";
import HistoryGroupItem from "./HistoryGroupItem";

type Props = {
  histories: IchHistory[];
};

export const HistoryGroup: React.FC<Props> = ({ histories }) => {
  return (
    <ul>
      {histories.map((h) => (
        <HistoryGroupItem key={h.id} history={h} />
      ))}
    </ul>
  );
};

export default HistoryGroup;
