<?php

$github_urls = ["alive.github.com",
    "live.github.com",
    "github.githubassets.com",
    "central.github.com",
    "desktop.githubusercontent.com",
    "assets-cdn.github.com",
    "camo.githubusercontent.com",
    "github.map.fastly.net",
    "github.global.ssl.fastly.net",
    "gist.github.com",
    "github.io",
    "github.com",
    "github.blog",
    "api.github.com",
    "raw.githubusercontent.com",
    "user-images.githubusercontent.com",
    "favicons.githubusercontent.com",
    "avatars5.githubusercontent.com",
    "avatars4.githubusercontent.com",
    "avatars3.githubusercontent.com",
    "avatars2.githubusercontent.com",
    "avatars1.githubusercontent.com",
    "avatars0.githubusercontent.com",
    "avatars.githubusercontent.com",
    "codeload.github.com",
    "github-cloud.s3.amazonaws.com",
    "github-com.s3.amazonaws.com",
    "github-production-release-asset-2e65be.s3.amazonaws.com",
    "github-production-user-asset-6210df.s3.amazonaws.com",
    "github-production-repository-file-5c1aeb.s3.amazonaws.com",
    "githubstatus.com",
    "github.community",
    "github.dev",
    "collector.github.com",
    "pipelines.actions.githubusercontent.com",
    "media.githubusercontent.com",
    "cloud.githubusercontent.com",
    "objects.githubusercontent.com"
];

$github_hosts = [];
$hosts_content = "# fetch-github-host begin\n";
foreach ($github_urls as $url) {
    $item = [gethostbyname($url), $url];
    $github_hosts[] = $item;
    $hosts_content .= str_pad($item[0], 28) . $item[1] . "\n";
}
$utc_date = date('c');
$hosts_content .= "# last fetch time: $utc_date\n# update url: https://hosts.gitcdn.top/hosts.txt\n# fetch-github-host end\n\n";

$template = file_get_contents('index-template.php');
file_put_contents('index.php', str_replace('<!--time-->', $utc_date, $template));
file_put_contents('hosts.txt', $hosts_content);
file_put_contents('hosts.json', json_encode($github_hosts));

echo "fetch success! ($utc_date)";





