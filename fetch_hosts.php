<?php

// 此文件为旧版本通过PHP获取Github Hosts，只能运用于服务端，不建议使用！

$github_urls = file_get_contents(dirname(__FILE__).'/domains.json');

$github_hosts = [];
$hosts_content = "# fetch-github-hosts begin\n";
foreach ($github_urls as $url) {
    $item = [gethostbyname($url), $url];
    $github_hosts[] = $item;
    $hosts_content .= str_pad($item[0], 28) . $item[1] . "\n";
}
$utc_date = date('c');
$hosts_content .= "# last fetch time: $utc_date\n# update url: https://hosts.gitcdn.top/hosts.txt\n# fetch-github-hosts end\n\n";

$template = file_get_contents('index-template.php');
file_put_contents('index.html', str_replace('<!--time-->', $utc_date, $template));
file_put_contents('hosts.txt', $hosts_content);
file_put_contents('hosts.json', json_encode($github_hosts));

echo "fetch success! ($utc_date)";





