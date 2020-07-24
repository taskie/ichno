import Head from "next/head";
import Link from "next/link";
import { applicationName } from "../config";
import { defaultInstance } from "../api/apiClient";
import { NextPage } from "next";
import { GetNamespacesResponse } from "@/api/types";
import NamespaceLink from "@/components/NamespaceLink";

type Response = GetNamespacesResponse;

type Props = { response?: Response; err?: string };

export const HomePage: NextPage<Props> = (props) => {
  return (
    <div className="container">
      <Head>
        <title>{applicationName}</title>
      </Head>
      <main>
        <h1 className="title">{applicationName}</h1>
        <h2>Namespaces</h2>
        <ul>
          {props.response?.namespaces.map((n) => (
            <li>
              <NamespaceLink key={n.id} namespaceId={n.id} family="stats" />
            </li>
          ))}
        </ul>
        <p>
          <Link href="/namespaces">
            <a>List</a>
          </Link>
        </p>
      </main>
    </div>
  );
};

HomePage.getInitialProps = async () => {
  try {
    const path = "namespaces";
    const { data } = await defaultInstance.get(path);
    return { response: data };
  } catch (err) {
    console.error(err);
    return { err: err.message };
  }
};

export default HomePage;
