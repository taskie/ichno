import { NextPage } from "next";
import Head from "next/head";
import { useRouter } from "next/router";
import { uria } from "@/utils/uri";
import { defaultInstance } from "@/api/apiClient";
import { applicationName } from "@/config";
import { GetDiffResponse } from "@/api/types";
import Digest from "@/components/Digest";
import StatGroup from "@/components/StatGroup";
import { useForm } from "react-hook-form";
import { useEffect } from "react";
import { rejectEmpty } from "@/utils/record";
import GlobalNav from "@/components/GlobalNav";
import FootprintLink from "@/components/FootprintLink";

type FormData = {
  group_name1?: string;
  path_prefix1?: string;
  group_name2?: string;
  path_prefix2?: string;
};

type Query = { workspaceName: string } & FormData;

type Response = GetDiffResponse;

type Props = { response?: Response; err?: string };

type DiffFormProps = {
  formData: FormData;
  onSubmit: (form: FormData) => void;
};

export const DiffForm: React.FC<DiffFormProps> = ({ onSubmit, formData }) => {
  const { register, handleSubmit, reset, getValues, setValue } = useForm<FormData>();
  useEffect(() => {
    reset(formData);
  }, [reset, formData]);
  const onUp = () => {
    const { path_prefix1, path_prefix2 } = getValues(["path_prefix1", "path_prefix2"]);
    if (path_prefix1 != null) {
      const splitted = path_prefix1.split("/");
      splitted.pop();
      setValue("path_prefix1", splitted.join("/"));
    }
    if (path_prefix2 != null) {
      const splitted = path_prefix2.split("/");
      splitted.pop();
      setValue("path_prefix2", splitted.join("/"));
    }
  };
  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <dl>
        <dt>
          <label>Group Name 1 / Path Prefix 1:</label>
        </dt>
        <dd>
          <input type="text" name="group_name1" placeholder="default" ref={register} />
          {" / "}
          <input type="text" name="path_prefix1" placeholder="data/archives" size={80} ref={register} />
        </dd>
        <dt>
          <label>Group Name 2 / Path Prefix 2:</label>
        </dt>
        <dd>
          <input type="text" name="group_name2" placeholder="default" ref={register} />
          {" / "}
          <input type="text" name="path_prefix2" placeholder="data/archives" size={80} ref={register} />
        </dd>
      </dl>
      <button type="button" onClick={onUp}>
        Up
      </button>
      <button>Select</button>
    </form>
  );
};

const ResponseView: React.FC<{ response: Response; workspaceName: string }> = ({
  workspaceName,
  response: { group1, group2, diff, stats, footprints },
}) => {
  return (
    <>
      <h2>Diff Table</h2>
      <table>
        <thead>
          <tr>
            <th>Digest</th>
            <th>Source</th>
            <th>Destination</th>
          </tr>
        </thead>
        <tbody>
          {Object.entries(diff).map(([k, [src, dst]]) => {
            const footprint = footprints[k];
            const srcStats = src.map((i) => stats[`${i}`]).filter((v) => v != null);
            const dstStats = dst.map((i) => stats[`${i}`]).filter((v) => v != null);
            return (
              <tr key={footprint.id}>
                <td>
                  <FootprintLink workspaceName={workspaceName} digest={footprint.digest} length={8} />
                </td>
                <td>
                  <StatGroup workspaceName={workspaceName} groupName={group1.name} stats={srcStats} mode="diff" />
                </td>
                <td>
                  <StatGroup workspaceName={workspaceName} groupName={group2.name} stats={dstStats} mode="diff" />
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </>
  );
};

export const DiffPage: NextPage<Props> = (props) => {
  const router = useRouter();
  const { query: rawQuery } = router;
  const { workspaceName, group_name1, path_prefix1, group_name2, path_prefix2 } = (rawQuery as unknown) as Query;
  const formData: FormData = { group_name1, path_prefix1, group_name2, path_prefix2 };
  const pageTitle = "Diff";
  const changeUrl = (data: FormData) => {
    const query = rejectEmpty(data);
    const href = { pathname: "/[workspaceName]/diff", query };
    const as = { pathname: uria`/${workspaceName}/diff`, query };
    router.push(href, as);
  };
  return (
    <div className="container">
      <Head>
        <title>
          {pageTitle} - {applicationName}
        </title>
      </Head>
      <GlobalNav workspaceName={workspaceName} />
      <h1>{pageTitle}</h1>
      <DiffForm formData={formData} onSubmit={changeUrl} />
      {props.response != null ? (
        <ResponseView response={props.response} workspaceName={workspaceName} />
      ) : props.err != null ? (
        <p>Some error occured: {props.err}</p>
      ) : undefined}
    </div>
  );
};

DiffPage.getInitialProps = async ({ query: rawQuery }) => {
  try {
    const { workspaceName, group_name1, path_prefix1, group_name2, path_prefix2 } = (rawQuery as unknown) as Query;
    if (group_name1 == null || path_prefix1 == null || group_name2 == null || path_prefix2 == null) {
      return { response: undefined };
    }
    const path = uria`${workspaceName}/diff`;
    const { data } = await defaultInstance.get(path, {
      params: { group_name1, path_prefix1, group_name2, path_prefix2 },
    });
    return { response: data };
  } catch (err) {
    // console.error(err);
    return { err: err.message };
  }
};

export default DiffPage;
