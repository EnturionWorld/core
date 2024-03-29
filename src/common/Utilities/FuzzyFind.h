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

#ifndef KITRON_FUZZYFIND_H
#define KITRON_FUZZYFIND_H

#include <map>
#include <string>
#include <type_traits>

namespace Kitron
{
    namespace Containers
    {
        template <typename Container, typename NeedleContainer, typename ContainsOperator = bool(std::string const&, std::string const&), typename T = void>
        auto FuzzyFindIn(Container const& container, NeedleContainer const& needles, ContainsOperator const& contains = StringContainsStringI, int(*bonus)(decltype((*std::begin(std::declval<Container>())))) = nullptr)
        {
            using IteratorResult = decltype((*std::begin(container)));
            using MappedType = std::conditional_t<std::is_reference_v<IteratorResult>, std::reference_wrapper<std::remove_reference_t<IteratorResult>>, IteratorResult>;
            std::multimap<size_t, MappedType, std::greater<size_t>> results;

            for (auto outerIt = std::begin(container), outerEnd = std::end(container); outerIt != outerEnd; ++outerIt)
            {
                size_t count = 0;
                for (auto innerIt = std::begin(needles), innerEnd = std::end(needles); innerIt != innerEnd; ++innerIt)
                    if (contains(*outerIt, *innerIt))
                        ++count;

                if (!count)
                    continue;

                if (bonus)
                    count += bonus(*outerIt);

                results.emplace(count, *outerIt);
            }

            return results;
        }
    }
}

#endif
