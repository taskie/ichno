import { IchStat } from "@/api/types";
import StatGroupItem from "./StatGroupItem";

type Props = {
  stats: IchStat[];
};

export const StatGroup: React.FC<Props> = ({ stats }) => {
  return (
    <ul>
      {stats.map((s) => (
        <StatGroupItem key={s.id} stat={s} />
      ))}
    </ul>
  );
};

export default StatGroup;
