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

#include "Log.h"
#include "Config.h"
#include "Errors.h"

Log::~Log() {
    if (_logMgr != nullptr)
        LogMgr_Free(_logMgr);
}

void Log::outMessage(std::string const& filter, LogLevel level, std::string&& message)
{
    if (_logMgr == nullptr) {
        return;
    }

    LogMgr_Write(_logMgr, filter.c_str(), level, message.c_str());
}

void Log::outCommand(std::string&& message)
{
    if (_logMgr == nullptr) {
        return;
    }

    LogMgr_Write(_logMgr, "commands.gm", LOG_LEVEL_INFO, message.c_str());
}

Log* Log::instance()
{
    static Log instance;
    return &instance;
}

void Log::Initialize(const struct Config* config)
{
    if (_logMgr != nullptr) {
        LogMgr_Free(_logMgr);
    }

    _logMgr = (LogMgr *)LogMgr_Initialize(config);
}
