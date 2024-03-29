/*
 * This file is part of the KitronCore Project. See AUTHORS file for Copyright information
 *
 * This program is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License as published by the
 * Free Software Foundation; either version 2 of the License, or (at your
 * option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
 * more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program. If not, see <http://www.gnu.org/licenses/>.
 */

#define CATCH_CONFIG_ENABLE_CHRONO_STRINGMAKER
#include "tc_catch2.h"

#include "Config.h"
#include <boost/filesystem.hpp>
#include <cstdlib>
#include <string>

std::string CreateConfigWithMap(std::map<std::string, std::string> const& map)
{
    auto mTempFileRel = boost::filesystem::unique_path("deleteme.ini");
    auto mTempFileAbs = boost::filesystem::temp_directory_path() / mTempFileRel;
    std::ofstream iniStream;
    iniStream.open(mTempFileAbs.c_str());

    for (auto const& itr : map)
        iniStream << itr.first << " = " << itr.second << "\n";

    iniStream.close();

    return mTempFileAbs.native();
}

TEST_CASE("Envariable variables", "[Config]")
{
    std::map<std::string, std::string> config;
    config["Int.Nested"] = "4242";
    config["lower"] = "simpleString";
    config["UPPER"] = "simpleString";
    config["SomeLong.NestedNameWithNumber.Like1"] = "1";

    auto filePath = CreateConfigWithMap(config);

    std::string err;
    REQUIRE(sConfigMgr->LoadInitial(filePath, err));
    REQUIRE(err.empty());

    SECTION("Nested int")
    {
        REQUIRE(sConfigMgr->GetIntDefault("Int.Nested", 10) == 4242);

        setenv("APP_INT__NESTED", "8080", 1);
        std::vector<std::string> configErrors;
        REQUIRE(sConfigMgr->Reload(configErrors));
        REQUIRE(sConfigMgr->GetIntDefault("Int.Nested", 10) == 8080);
    }

    SECTION("Simple lower string")
    {
        REQUIRE(sConfigMgr->GetStringDefault("lower", "") == "simpleString");

        setenv("APP_LOWER", "envstring", 1);
        std::vector<std::string> configErrors;
        REQUIRE(sConfigMgr->Reload(configErrors));
        REQUIRE(sConfigMgr->GetStringDefault("lower", "") == "envstring");
    }

    SECTION("Simple upper string")
    {
        REQUIRE(sConfigMgr->GetStringDefault("UPPER", "") == "simpleString");

        setenv("APP_UPPER", "envupperstring", 1);
        std::vector<std::string> configErrors;
        REQUIRE(sConfigMgr->Reload(configErrors));
        REQUIRE(sConfigMgr->GetStringDefault("UPPER", "") == "envupperstring");
    }

    SECTION("Long nested name with number")
    {
        REQUIRE(sConfigMgr->GetFloatDefault("SomeLong.NestedNameWithNumber.Like1", 0) == 1);

        setenv("APP_SOME_LONG__NESTED_NAME_WITH_NUMBER__LIKE_1", "42", 1);
        std::vector<std::string> configErrors;
        REQUIRE(sConfigMgr->Reload(configErrors));
        REQUIRE(sConfigMgr->GetFloatDefault("SomeLong.NestedNameWithNumber.Like1", 0) == 42);
    }

    SECTION("String that not exist in config")
    {
        setenv("APP_UNIQUE__STRING", "somevalue", 1);
        std::vector<std::string> configErrors;
        REQUIRE(sConfigMgr->Reload(configErrors));
        REQUIRE(sConfigMgr->GetStringDefault("Unique.String", "") == "somevalue");
    }

    SECTION("Int that not exist in config")
    {
        setenv("APP_UNIQUE__INT", "100", 1);
        std::vector<std::string> configErrors;
        REQUIRE(sConfigMgr->Reload(configErrors));
        REQUIRE(sConfigMgr->GetIntDefault("Unique.Int", 1) == 100);
    }

    SECTION("Not existing string")
    {
        REQUIRE(sConfigMgr->GetStringDefault("NotFound.String", "none") == "none");
    }

    SECTION("Not existing int")
    {
        REQUIRE(sConfigMgr->GetIntDefault("NotFound.Int", 1) == 1);
    }

    std::remove(filePath.c_str());
}
