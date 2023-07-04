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

#ifndef KITRONCORE_LOG_H
#define KITRONCORE_LOG_H

#include "Define.h"
#include "LogCommon.h"
#include "StringFormat.h"
#include "libenturion_shared.h"

class TC_COMMON_API Log
{
    struct LogMgr *_logMgr;
    Log(): _logMgr(nullptr) {
    }

    ~Log();

    public:
        Log(Log const&) = delete;
        Log(Log&&) = delete;
        Log& operator=(Log const&) = delete;
        Log& operator=(Log&&) = delete;

        static Log* instance();

        void Initialize(const struct Config* config);

        template<typename Format, typename... Args>
        inline void outMessage(std::string const& filter, LogLevel const level, Format&& fmt, Args&&... args)
        {
            outMessage(filter, level, Kitron::StringFormat(std::forward<Format>(fmt), std::forward<Args>(args)...));
        }

        template<typename Format, typename... Args>
        void outCommand(uint32 account, Format&& fmt, Args&&... args)
        {
            outCommand(Kitron::StringFormat(std::forward<Format>(fmt), std::forward<Args>(args)...), std::to_string(account));
        }

        void SetRealmId(uint32 id);

    private:
        void outMessage(std::string const& filter, LogLevel level, std::string&& message);
        void outCommand(std::string&& message, std::string&& param1);
};

#define sLog Log::instance()

#define LOG_EXCEPTION_FREE(filterType__, level__, ...) \
    sLog->outMessage(filterType__, level__, __VA_ARGS__)

#ifdef PERFORMANCE_PROFILING
#define TC_LOG_MESSAGE_BODY(filterType__, level__, ...) ((void)0)
#elif KITRON_PLATFORM != KITRON_PLATFORM_WINDOWS
#define TC_LOG_MESSAGE_BODY(filterType__, level__, ...)        \
        LOG_EXCEPTION_FREE(filterType__, level__, __VA_ARGS__)
#else
#define TC_LOG_MESSAGE_BODY(filterType__, level__, ...)        \
        __pragma(warning(push))                                \
        __pragma(warning(disable:4127))                        \
        LOG_EXCEPTION_FREE(filterType__, level__, __VA_ARGS__) \
        __pragma(warning(pop))
#endif

#define TC_LOG_TRACE(filterType__, ...) \
    TC_LOG_MESSAGE_BODY(filterType__, LOG_LEVEL_TRACE, __VA_ARGS__)

#define TC_LOG_DEBUG(filterType__, ...) \
    TC_LOG_MESSAGE_BODY(filterType__, LOG_LEVEL_DEBUG, __VA_ARGS__)

#define TC_LOG_INFO(filterType__, ...)  \
    TC_LOG_MESSAGE_BODY(filterType__, LOG_LEVEL_INFO, __VA_ARGS__)

#define TC_LOG_WARN(filterType__, ...)  \
    TC_LOG_MESSAGE_BODY(filterType__, LOG_LEVEL_WARN, __VA_ARGS__)

#define TC_LOG_ERROR(filterType__, ...) \
    TC_LOG_MESSAGE_BODY(filterType__, LOG_LEVEL_ERROR, __VA_ARGS__)

#endif
