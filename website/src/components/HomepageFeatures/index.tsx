import clsx from "clsx";
import Heading from "@theme/Heading";
import styles from "./styles.module.css";

type FeatureItem = {
  title: string;
  SVG: React.ComponentType<React.SVGProps<SVGSVGElement>>;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: "Browser Recording",
    SVG: () => (
      <svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
        <rect
          x="20"
          y="30"
          width="160"
          height="120"
          rx="8"
          fill="none"
          stroke="currentColor"
          strokeWidth="4"
        />
        <rect
          x="30"
          y="40"
          width="140"
          height="90"
          fill="currentColor"
          opacity="0.1"
        />
        <circle cx="100" cy="170" r="15" fill="none" stroke="currentColor" strokeWidth="4" />
      </svg>
    ),
    description: (
      <>
        Records Playwright browser sessions with full telemetry capture.
        Mouse movements, clicks, and zoom levels are tracked automatically.
      </>
    ),
  },
  {
    title: "Zoom Effects",
    SVG: () => (
      <svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
        <circle
          cx="100"
          cy="100"
          r="60"
          fill="none"
          stroke="currentColor"
          strokeWidth="4"
        />
        <circle
          cx="100"
          cy="100"
          r="30"
          fill="currentColor"
          opacity="0.2"
        />
        <line x1="145" y1="145" x2="180" y2="180" stroke="currentColor" strokeWidth="6" strokeLinecap="round" />
      </svg>
    ),
    description: (
      <>
        Smooth zoom transitions between captured points.
        Emphasize important interactions in your demo videos.
      </>
    ),
  },
  {
    title: "Cross-Platform",
    SVG: () => (
      <svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
        <rect x="20" y="50" width="80" height="60" rx="4" fill="none" stroke="currentColor" strokeWidth="4" />
        <rect x="110" y="30" width="70" height="90" rx="4" fill="none" stroke="currentColor" strokeWidth="4" />
        <rect x="30" y="130" width="140" height="40" rx="4" fill="none" stroke="currentColor" strokeWidth="4" />
      </svg>
    ),
    description: (
      <>
        Pre-built binaries for Linux, macOS, and Windows.
        No build tools required for your users.
      </>
    ),
  },
];

function Feature({ title, SVG, description }: FeatureItem) {
  return (
    <div className={clsx("col col--4")}>
      <div className="text--center">
        <SVG className={styles.featureSvg} />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
