REVOKE ALL PRIVILEGES ON * . * FROM 'kitron'@'localhost';

REVOKE ALL PRIVILEGES ON `world` . * FROM 'kitron'@'localhost';

REVOKE GRANT OPTION ON `world` . * FROM 'kitron'@'localhost';

REVOKE ALL PRIVILEGES ON `characters` . * FROM 'kitron'@'localhost';

REVOKE GRANT OPTION ON `characters` . * FROM 'kitron'@'localhost';

REVOKE ALL PRIVILEGES ON `auth` . * FROM 'kitron'@'localhost';

REVOKE GRANT OPTION ON `auth` . * FROM 'kitron'@'localhost';

DROP USER 'kitron'@'localhost';

DROP DATABASE IF EXISTS `world`;

DROP DATABASE IF EXISTS `characters`;

DROP DATABASE IF EXISTS `auth`;
